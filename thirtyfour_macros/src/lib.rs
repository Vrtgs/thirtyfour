extern crate proc_macro;
use itertools::izip;
use proc_macro::TokenStream;
use proc_macro2::Literal;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericArgument, Lit, Meta, MetaNameValue, NestedMeta,
    Path, PathArguments, PathSegment, Type,
};

/// Derive macro for a wrapped `Component`.
///
/// A `Component` contains a base [`WebElement`] from which all element queries will be performed.
///
/// All elements in the component are descendents of the base element (or at least the
/// starting point for an element query, since XPath queries can access parent nodes).
///
/// Components perform element lookups via [`ElementResolver`]s, which lazily perform an
/// element query to resolve a [`WebElement`], and then cache the result for later access.
///
/// See the [`ElementResolver`] documentation for more details.
///
/// ## Attributes
///
/// ### `#[base]`
/// By default the base element should be named `base` and be of type `WebElement`.
/// You can optionally use the `#[base]` attribute if you wish to name the base element
/// something other than `base`. If you use this attribute you cannot also have another
/// element named `base`.
///
/// ### `#[by(..)]`
/// Components use the `#[by(..)]` attribute to specify all of the details of the query.
///
/// The only required attribute is the selector attribute, which can be one of the following:
/// - `id = "..."`: Select element by id.
/// - `tag = "..."`: Select element by tag name.
/// - `link = "..."`: Select element by link text.
/// - `css = "..."`: Select element by CSS.
/// - `xpath = "..."`: Select element by XPath.
/// - `name = "..."`: Select element by name.
/// - `class = "..."`: Select element by class name.
///
/// Optional attributes available within `#[by(..)]` include:
/// - `first`: (single element only) Select the first element that matches the query.
///            By default a query will return `NoSuchElement` if multiple elements match.
///            This default is designed to catch instances where a query is not specific enough.
/// - `allow_empty`: (multi elements only) Return an empty Vec if no elements were found.
///                  By default a multi-element query will return `NoSuchElement` if no
///                  elements were found.
/// - `description = "..."`: Set the element description to be displayed in `NoSuchElement` errors.
/// - `allow_errors`: Ignore errors such as stale elements while polling.
/// - `wait(timeout_ms = 10000, interval_ms=500)`: Override the default polling options.
/// - `nowait`: Turn off polling for this element query.
/// - `custom = "func"`: Use the specified [`ElementQueryFn<T>`] function.
///                      The type `T` for your `ElementQueryFn<T>` function corresponds to
///                      the type `T` in `ElementResolver<T>`. Supported types are
///                      `WebElement`, `Vec<WebElement`, `T: Component` and `Vec<T: Component>`.
///                      **NOTE**: The `custom` attribute cannot be specified with any other
///                      attribute.
///
/// See [`ElementQueryOptions`] for more details on how each option is used.
///
/// ## Example:
/// ```ignore
/// /// This component shows how to nest components inside others.
/// #[derive(Debug, Clone, Component)]
/// pub struct CheckboxSectionComponent {
///     base: WebElement,
///     #[by(tag = "label", allow_empty)]
///     boxes: ElementResolver<Vec<CheckboxComponent>>,
///     // Other fields will be initialised using `Default::default()`.
///     my_field: bool,
/// }
///
/// /// This component shows how to wrap a simple web component.
/// #[derive(Debug, Clone, Component)]
/// pub struct CheckboxComponent {
///     base: WebElement,
///     #[by(css = "input[type='checkbox']", first)]
///     input: ElementResolver<WebElement>,
///     #[by(name = "text-label", description = "text label")]
///     label: ElementResolver<WebElement>
/// }
///
/// impl CheckboxComponent {
///     /// Return true if the checkbox is ticked.
///     pub async fn is_ticked(&self) -> WebDriverResult<bool> {
///         // Equivalent to: let elem = self.input.resolve_present().await?;
///         let elem = resolve_present!(self.input);
///         let prop = elem.prop("checked").await?;
///         Ok(prop.unwrap_or_default() == "true")
///     }
/// }
/// ```
/// [`WebElement`]: thirtyfour::WebElement
/// [`ElementResolver`]: thirtyfour::components::ElementResolver
/// [`ElementQueryOptions`]: thirtyfour::extensions::query::ElementQueryOptions
#[proc_macro_derive(Component, attributes(base, by))]
#[proc_macro_error::proc_macro_error]
pub fn derive_component_fn(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = ast.ident;

    let (base, prefields, fields) = match ast.data {
        Data::Struct(s) => {
            match s.fields {
                Fields::Named(nf) => {
                    // First we parse the field names and any field attributes.
                    let field_names = nf.named.iter().map(|x| {
                        x.ident.as_ref().unwrap_or_else(|| abort!(x, "unknown field name"))
                    });
                    let field_types = nf.named.iter().map(|x| &x.ty);
                    let field_attrs = nf.named.iter().map(|x| &x.attrs);

                    // We split into fields and "prefields" so that in the `new()` method
                    // we can do:
                    // ```ignore
                    // let field = ElementResolver::new(); // <- this is the `prefields` line.
                    //
                    // Self {
                    //     base: base,
                    //     field, // <- This is the `fields` line.
                    // }
                    // ```
                    let mut fields = Vec::new();
                    let mut prefields = Vec::new();
                    let mut base_field = None;

                    // Now we construct the field initialisers.
                    for (field_name, field_type, attrs) in
                        izip!(field_names, field_types, field_attrs)
                    {
                        // Find base element.
                        let is_base = attrs.iter().any(|x| x.path.is_ident("base"));
                        if (base_field.is_none() && field_name == "base") || is_base {
                            match field_type {
                                Type::Path(p) => {
                                    if !p.path.is_ident("WebElement") {
                                        abort! { p, "base field must be a WebElement" }
                                    }
                                }
                                t => abort! { t, "base field must be a WebElement" },
                            }
                            base_field = Some(field_name.clone());
                            continue;
                        }

                        // Get "by" attribute for field, if any.
                        let mut by_ident = None;
                        for attr in attrs {
                            if attr.path.is_ident("by") {
                                if let Ok(x) = ByTokens::try_from(attr) {
                                    by_ident = Some(x);
                                }
                            }
                        }

                        // Initializer
                        let (predef, def) = match field_type {
                            Type::Path(p) => {
                                match by_ident {
                                    Some(by) => {
                                        // Has a #[by()] attribute.
                                        if by.is_multi() || is_multi_resolver(&p.path) {
                                            // Multi-element resolver.
                                            let multi_args: MultiResolverArgs = by.into();
                                            let multi_constructor: proc_macro2::TokenStream =
                                                multi_args.into();

                                            let ty = fix_type(p.path.clone());

                                            let predef = quote! {
                                                let #field_name = #ty::#multi_constructor
                                            };
                                            let def = quote! {
                                                #field_name
                                            };
                                            (Some(predef), def)
                                        } else {
                                            // Single-element resolver.
                                            let single_args: SingleResolverArgs = by.into();
                                            let single_constructor: proc_macro2::TokenStream =
                                                single_args.into();

                                            let ty = fix_type(p.path.clone());

                                            let predef = quote! {
                                                let #field_name = #ty::#single_constructor
                                            };
                                            let def = quote! {
                                                #field_name
                                            };
                                            (Some(predef), def)
                                        }
                                    }
                                    _ => {
                                        // No #[by()] attribute.
                                        let def = quote! {
                                            # field_name: Default::default()
                                        };
                                        (None, def)
                                    }
                                }
                            }
                            _ => {
                                // Some other field type.
                                let def = quote! {
                                    #field_name: Default::default()
                                };
                                (None, def)
                            }
                        };

                        if let Some(pre) = predef {
                            prefields.push(pre);
                        }

                        fields.push(def);
                    }
                    (base_field, prefields, fields)
                }
                _ => panic!("Tuple or unit structs not supported"),
            }
        }
        Data::Enum(_) | Data::Union(_) => {
            panic!("Component attribute not supported for enums or unions")
        }
    };
    let base = base.unwrap_or_else(|| {
        abort!(
            ident,
            "base field not found. Add the #[base] attribute for the base WebElement field"
        )
    });

    // Now generate the code.
    let gen = quote! {
        impl #ident {
            pub fn new(base: thirtyfour::WebElement) -> Self {
                #(#prefields)*
                Self {
                    #base: base,
                    #(#fields,)*
                }
            }
        }

        #[automatically_derived]
        impl From<thirtyfour::WebElement> for #ident {
            fn from(elem: thirtyfour::WebElement) -> Self {
                Self::new(elem)
            }
        }

        #[automatically_derived]
        impl Component for #ident {
            fn base_element(&self) -> thirtyfour::WebElement {
                self.#base.clone()
            }
        }
    };
    gen.into()
}

#[derive(Debug, Clone)]
struct WaitOptions {
    timeout_ms: Literal,
    interval_ms: Literal,
}

/// These are all of the supported tokens in a `#[by(..)]` attribute.
#[derive(Debug)]
enum ByToken {
    Id(Literal),
    Tag(Literal),
    LinkText(Literal),
    Css(Literal),
    XPath(Literal),
    Name(Literal),
    ClassName(Literal),
    Multi,
    AllowEmpty,
    First,
    IgnoreErrors,
    Description(Literal),
    Wait(WaitOptions),
    NoWait,
    CustomFn(String),
}

impl ByToken {
    /// Helper for making sure the right things are mutually exclusive.
    ///
    /// This is how we catch repeated options.
    fn get_unique_type(&self) -> &str {
        match &self {
            ByToken::Id(_)
            | ByToken::Tag(_)
            | ByToken::LinkText(_)
            | ByToken::Css(_)
            | ByToken::XPath(_)
            | ByToken::Name(_)
            | ByToken::ClassName(_) => "selector",
            ByToken::Multi => "multi",
            ByToken::AllowEmpty => "allow_empty",
            ByToken::First => "first",
            ByToken::IgnoreErrors => "ignore_errors",
            ByToken::Description(_) => "description",
            ByToken::Wait(_) => "wait",
            ByToken::NoWait => "nowait",
            ByToken::CustomFn(_) => "custom",
        }
    }

    /// Get the above unique names that are not supported with each token.
    ///
    /// This is how mutually exclusive options are determined.
    fn get_disallowed_types(&self) -> Vec<&str> {
        match &self {
            ByToken::First => vec!["multi", "custom"],
            ByToken::AllowEmpty | ByToken::IgnoreErrors | ByToken::Description(_) => vec!["custom"],
            ByToken::Wait(_) => vec!["custom", "nowait"],
            ByToken::NoWait => vec!["custom", "wait"],
            ByToken::CustomFn(_) => {
                vec![
                    "multi",
                    "first",
                    "ignore_errors",
                    "description",
                    "wait",
                    "nowait",
                    "allow_empty",
                ]
            }
            _ => vec![],
        }
    }
}

/// Convert `Meta` into `ByToken`.
///
/// This is where all tokens are parsed into `ByToken` variants.
impl TryFrom<Meta> for ByToken {
    type Error = TokenStream;

    fn try_from(value: Meta) -> Result<Self, Self::Error> {
        match value {
            Meta::Path(p) => match p {
                k if k.is_ident("multi") => Ok(ByToken::Multi),
                k if k.is_ident("allow_empty") => Ok(ByToken::AllowEmpty),
                k if k.is_ident("first") => Ok(ByToken::First),
                k if k.is_ident("ignore_errors") => Ok(ByToken::IgnoreErrors),
                k if k.is_ident("nowait") => Ok(ByToken::NoWait),
                e => abort! { e, format!("unknown attribute {e:?}") },
            },
            Meta::List(l) => match l.path {
                // wait(timeout_ms = u32, interval_ms = u32)
                p if p.is_ident("wait") => {
                    let mut timeout: Option<Literal> = None;
                    let mut interval: Option<Literal> = None;
                    for n in l.nested.into_iter() {
                        match n {
                            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                path,
                                lit,
                                ..
                            })) => match (path, lit) {
                                (k, Lit::Int(v)) if k.is_ident("timeout_ms") => {
                                    assert!(timeout.is_none(), "cannot specify timeout twice");
                                    timeout = Some(v.token());
                                }
                                (k, Lit::Int(v)) if k.is_ident("interval_ms") => {
                                    assert!(interval.is_none(), "cannot specify interval twice");
                                    interval = Some(v.token());
                                }
                                e => {
                                    abort! { p , format!("unknown attribute {e:?} (must be timeout_ms or interval_ms)") }
                                }
                            },
                            e => {
                                abort! { p, format!("unknown attribute {e:?} (format should be `wait(timeout_ms=30000, interval_ms=500)`)") }
                            }
                        }
                    }

                    match (timeout, interval) {
                        (Some(t), Some(i)) => Ok(ByToken::Wait(WaitOptions {
                            timeout_ms: t,
                            interval_ms: i,
                        })),
                        _ => {
                            abort! { p, "wait attribute requires the following args: timeout_ms, interval_ms" }
                        }
                    }
                }
                e => abort! { e, format!("unknown attribute: {e:?}") },
            },
            Meta::NameValue(MetaNameValue {
                path,
                lit,
                ..
            }) => match (path, lit) {
                (k, Lit::Str(v)) if k.is_ident("id") => Ok(ByToken::Id(v.token())),
                (k, Lit::Str(v)) if k.is_ident("tag") => Ok(ByToken::Tag(v.token())),
                (k, Lit::Str(v)) if k.is_ident("link") => Ok(ByToken::LinkText(v.token())),
                (k, Lit::Str(v)) if k.is_ident("css") => Ok(ByToken::Css(v.token())),
                (k, Lit::Str(v)) if k.is_ident("xpath") => Ok(ByToken::XPath(v.token())),
                (k, Lit::Str(v)) if k.is_ident("name") => Ok(ByToken::Name(v.token())),
                (k, Lit::Str(v)) if k.is_ident("class") => Ok(ByToken::ClassName(v.token())),
                (k, Lit::Str(v)) if k.is_ident("description") => {
                    Ok(ByToken::Description(v.token()))
                }
                (k, Lit::Str(v)) if k.is_ident("custom") => Ok(ByToken::CustomFn(v.value())),
                (k, ..) => abort! { k, format!("unknown attribute: {k:?}") },
            },
        }
    }
}

/// Wrapper for a list of tokens so we can add methods to it.
struct ByTokens {
    tokens: Vec<ByToken>,
}

impl ByTokens {
    /// Apply validation rules to the list of tokens.
    ///
    /// This is where we determine whether these tokens are compatible with each other.
    pub fn validate(&self) -> Result<(), String> {
        let mut unique_tokens = HashSet::new();
        for token in self.tokens.iter() {
            let t = token.get_unique_type();
            if unique_tokens.contains(t) {
                return Err(format!("duplicate token '{t}' (cannot specify multiple)"));
            }
            unique_tokens.insert(t);
        }
        for token in self.tokens.iter() {
            let disallowed = token.get_disallowed_types();
            for t in disallowed {
                if unique_tokens.contains(t) {
                    let unique = token.get_unique_type();
                    return Err(format!("cannot specify '{unique}' with '{t}'"));
                }
            }
        }

        Ok(())
    }

    /// Extract just the "By"-specific part of the tokens.
    ///
    /// For example, `name = "element-name"`.
    ///
    /// This removes the token from the vec so that we can check for leftover tokens afterwards.
    ///
    /// This will also panic if more than one such token exists.
    pub fn take_quote(&mut self) -> proc_macro2::TokenStream {
        let mut ret = Vec::new();
        let tokens_in = std::mem::take(&mut self.tokens);
        for token in tokens_in.into_iter() {
            match token {
                ByToken::Id(id) => ret.push(quote! { By::Id(#id) }),
                ByToken::Tag(tag) => ret.push(quote! { By::Tag(#tag) }),
                ByToken::LinkText(text) => ret.push(quote! { By::LinkText(#text) }),
                ByToken::Css(css) => ret.push(quote! { By::Css(#css) }),
                ByToken::XPath(xpath) => ret.push(quote! { By::XPath(#xpath) }),
                ByToken::Name(name) => ret.push(quote! { By::Name(#name) }),
                ByToken::ClassName(class_name) => ret.push(quote! { By::ClassName(#class_name) }),
                t => self.tokens.push(t),
            }
        }

        match ret.len() {
            0 => panic!("no selector found"),
            1 => ret.into_iter().next().unwrap(),
            _ => panic!("multiple selectors are not supported"),
        }
    }

    pub fn is_multi(&self) -> bool {
        self.tokens.iter().any(|x| matches!(&x, ByToken::Multi))
    }

    pub fn take_one<F, T>(&mut self, f: F) -> Option<T>
    where
        F: Fn(&ByToken) -> Option<T>,
    {
        let mut pos = None;
        let mut value = None;
        for (i, t) in self.tokens.iter().enumerate() {
            if let Some(v) = f(t) {
                pos = Some(i);
                value = Some(v);
                break;
            }
        }

        match (pos, value) {
            (Some(p), Some(v)) => {
                self.tokens.remove(p);
                Some(v)
            }
            _ => None,
        }
    }

    pub fn take_multi(&mut self) -> Option<bool> {
        self.take_one(|x| match x {
            ByToken::Multi => Some(true),
            _ => None,
        })
    }

    pub fn take_first(&mut self) -> Option<bool> {
        self.take_one(|x| match x {
            ByToken::First => Some(true),
            _ => None,
        })
    }

    pub fn take_allow_empty(&mut self) -> Option<bool> {
        self.take_one(|x| match x {
            ByToken::AllowEmpty => Some(true),
            _ => None,
        })
    }

    pub fn take_ignore_errors(&mut self) -> Option<bool> {
        self.take_one(|x| match x {
            ByToken::IgnoreErrors => Some(true),
            _ => None,
        })
    }

    pub fn take_description(&mut self) -> Option<Literal> {
        self.take_one(|x| match x {
            ByToken::Description(d) => Some(d.clone()),
            _ => None,
        })
    }

    pub fn take_wait_options(&mut self) -> Option<WaitOptions> {
        self.take_one(|x| match x {
            ByToken::Wait(w) => Some(w.clone()),
            _ => None,
        })
    }

    pub fn take_nowait(&mut self) -> Option<bool> {
        self.take_one(|x| match x {
            ByToken::NoWait => Some(true),
            _ => None,
        })
    }

    pub fn take_custom(&mut self) -> Option<String> {
        self.take_one(|x| match x {
            ByToken::CustomFn(f) => Some(f.clone()),
            _ => None,
        })
    }
}

/// Parse an attribute into tokens.
///
/// This uses the above `TryFrom` impl to parse each `Meta` into `ByToken` variants.
impl TryFrom<&Attribute> for ByTokens {
    type Error = TokenStream;

    fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
        let meta = attr.parse_meta().expect("invalid arg format");
        let mut by_tokens = ByTokens {
            tokens: Vec::new(),
        };
        match meta {
            Meta::List(l) => {
                if !l.path.is_ident("by") {
                    abort!(l, "only 'by' attributes are supported here");
                }
                let args: Vec<NestedMeta> = l.nested.into_iter().collect();
                for arg in &args {
                    let token = match arg {
                        NestedMeta::Meta(meta) => ByToken::try_from(meta.clone())?,
                        t => {
                            abort! { t, format!("unrecognised token: {t:?}") }
                        }
                    };
                    by_tokens.tokens.push(token);
                    by_tokens.validate().unwrap_or_else(|e| {
                        abort! { arg , format!("{e}")}
                    });
                }
            }
            _ => panic!("unrecognised by argument format"),
        }

        Ok(by_tokens)
    }
}

/// Return true if this path should be treated as a multi-element resolver.
///
/// Basically any `ElementResolver<Vec<T>>` should be treated as multi.
///
/// We also catch `ElementResolverMulti` as a special case.
///
/// NOTE: If you use your own type alias for a multi-element resolver, you will need
///       to specify the `multi` attribute to force it to be treated as multi-element.
fn is_multi_resolver(path: &Path) -> bool {
    // First check for the type alias.
    if path.is_ident("ElementResolverMulti") {
        true
    } else {
        if let Some(x) = path.segments.last() {
            if x.ident == "ElementResolver" {
                // If we have `ElementResolver<Vec<T>>` then use multi.
                if let PathArguments::AngleBracketed(x) = &x.arguments {
                    for arg in &x.args {
                        if let GenericArgument::Type(Type::Path(t)) = arg {
                            let idents_of_path =
                                t.path.segments.iter().fold(String::new(), |mut acc, v| {
                                    acc.push_str(&v.ident.to_string());
                                    acc.push(':');
                                    acc
                                });

                            return ["Vec:", "vec:Vec:", "std:vec:Vec:", "alloc:vec:Vec:"]
                                .into_iter()
                                .any(|x| idents_of_path == x);
                        }
                    }
                }
            }
        }

        false
    }
}

/// All args for a single element resolver.
enum SingleResolverArgs {
    CustomFn(String),
    Opts {
        by: proc_macro2::TokenStream,
        first: Option<bool>,
        ignore_errors: Option<bool>,
        description: Option<Literal>,
        wait: Option<WaitOptions>,
        nowait: Option<bool>,
    },
}

/// First we convert `ByTokens` to `SingleResolverArgs`.
impl From<ByTokens> for SingleResolverArgs {
    fn from(mut t: ByTokens) -> Self {
        let s = match t.take_custom() {
            Some(f) => Self::CustomFn(f),
            None => Self::Opts {
                by: t.take_quote(),
                first: t.take_first(),
                ignore_errors: t.take_ignore_errors(),
                description: t.take_description(),
                wait: t.take_wait_options(),
                nowait: t.take_nowait(),
            },
        };

        assert!(t.tokens.is_empty(), "unrecognised args: {:?}", t.tokens);
        s
    }
}

/// Then we convert `SingleResolverArgs` to `TokenStream`.
impl From<SingleResolverArgs> for proc_macro2::TokenStream {
    fn from(args: SingleResolverArgs) -> Self {
        match args {
            SingleResolverArgs::CustomFn(f) => {
                let f_ident = format_ident!("{f}");
                quote! {
                    new_custom(base.clone(), #f_ident);
                }
            }
            SingleResolverArgs::Opts {
                by,
                first,
                ignore_errors,
                description,
                wait,
                nowait,
            } => {
                let ignore_errors_ident = match ignore_errors {
                    Some(true) => {
                        quote! { Some(true) }
                    }
                    _ => quote! { None },
                };
                let description_ident = match description {
                    Some(desc) => {
                        quote! { Some(#desc.to_string()) }
                    }
                    None => quote! { None },
                };
                let wait_ident = match wait {
                    Some(WaitOptions {
                        timeout_ms,
                        interval_ms,
                    }) => {
                        quote! {
                            Some(thirtyfour::extensions::query::ElementQueryWaitOptions::Wait {
                                timeout: std::time::Duration::from_millis(#timeout_ms),
                                interval: std::time::Duration::from_millis(#interval_ms)
                            })
                        }
                    }
                    None => match nowait {
                        Some(true) => {
                            quote! {
                                Some(thirtyfour::extensions::query::ElementQueryWaitOptions::NoWait)
                            }
                        }
                        _ => quote! { None },
                    },
                };
                let opts_ident = quote! {
                    thirtyfour::extensions::query::ElementQueryOptions::default()
                        .set_ignore_errors(#ignore_errors_ident)
                        .set_description::<String>(#description_ident)
                        .set_wait(#wait_ident)
                };

                match first {
                    Some(true) => {
                        quote! {
                            new_first_opts(base.clone(), #by, #opts_ident);
                        }
                    }
                    _ => {
                        quote! {
                            new_single_opts(base.clone(), #by, #opts_ident);
                        }
                    }
                }
            }
        }
    }
}

/// All args for a multi-element resolver.
enum MultiResolverArgs {
    CustomFn(String),
    Opts {
        by: proc_macro2::TokenStream,
        allow_empty: Option<bool>,
        ignore_errors: Option<bool>,
        description: Option<Literal>,
        wait: Option<WaitOptions>,
        nowait: Option<bool>,
    },
}

/// First we convert `ByTokens` into `MultiResolverArgs`.
impl From<ByTokens> for MultiResolverArgs {
    fn from(mut t: ByTokens) -> Self {
        t.take_multi(); // Not used here.
        let s = match t.take_custom() {
            Some(f) => Self::CustomFn(f),
            None => Self::Opts {
                by: t.take_quote(),
                allow_empty: t.take_allow_empty(),
                ignore_errors: t.take_ignore_errors(),
                description: t.take_description(),
                wait: t.take_wait_options(),
                nowait: t.take_nowait(),
            },
        };

        assert!(t.tokens.is_empty(), "unrecognised args: {:?}", t.tokens);
        s
    }
}

/// Then we convert `MultiResolverArgs` into `TokenStream`.
impl From<MultiResolverArgs> for proc_macro2::TokenStream {
    fn from(args: MultiResolverArgs) -> Self {
        match args {
            MultiResolverArgs::CustomFn(f) => {
                let f_ident = format_ident!("{f}");
                quote! {
                    new_custom(base.clone(), #f_ident);
                }
            }
            MultiResolverArgs::Opts {
                by,
                allow_empty,
                ignore_errors,
                description,
                wait,
                nowait,
            } => {
                let ignore_errors_ident = match ignore_errors {
                    Some(true) => {
                        quote! { Some(true) }
                    }
                    _ => quote! { None },
                };
                let description_ident = match description {
                    Some(desc) => quote! { Some(#desc.to_string()) },
                    None => quote! {
                        None,
                    },
                };
                let wait_ident = match wait {
                    Some(WaitOptions {
                        timeout_ms,
                        interval_ms,
                    }) => {
                        quote! {
                            Some(thirtyfour::extensions::query::ElementQueryWaitOptions::Wait {
                                timeout: std::time::Duration::from_millis(#timeout_ms),
                                interval: std::time::Duration::from_millis(#interval_ms)
                            })
                        }
                    }
                    None => match nowait {
                        Some(true) => {
                            quote! {
                                Some(thirtyfour::extensions::query::ElementQueryWaitOptions::NoWait)
                            }
                        }
                        _ => quote! { None },
                    },
                };
                let opts_ident = quote! {
                    thirtyfour::extensions::query::ElementQueryOptions::default()
                        .set_ignore_errors(#ignore_errors_ident)
                        .set_description::<String>(#description_ident)
                        .set_wait(#wait_ident)
                };

                match allow_empty {
                    Some(true) => {
                        quote! {
                            new_allow_empty_opts(base.clone(), #by, #opts_ident);
                        }
                    }
                    _ => {
                        quote! {
                            new_not_empty_opts(base.clone(), #by, #opts_ident);
                        }
                    }
                }
            }
        }
    }
}

/// Converts GenericType<Args> to GenericType::<Args> in order to call ::new_*() on it.
///
/// Non-generic types will be returned as is.
fn fix_type(mut ty: Path) -> proc_macro2::TokenStream {
    let last = ty.segments.pop();
    match last {
        Some(pair) => {
            let (p, _) = pair.into_tuple();
            let ident = p.ident;
            let args = p.arguments;
            if args.is_empty() {
                ty.segments.push(PathSegment::from(ident));
                quote! { #ty }
            } else if ty.segments.is_empty() {
                quote! { #ident::# args }
            } else {
                quote! { #ty::#ident::#args }
            }
        }
        None => {
            quote! {}
        }
    }
}
