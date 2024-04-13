/// Code for starting a new session.
pub mod create;
/// The underlying session handle.
pub mod handle;
/// HTTP helpers for WebDriver commands.
pub mod http;
/// Helper for values returned from scripts.
pub mod scriptret;

mod sealed {
    use std::borrow::Cow;
    use std::rc::Rc;
    use std::sync::Arc;

    pub trait IntoTransfer {
        fn into(self) -> Arc<str>;
    }

    impl IntoTransfer for &str {
        fn into(self) -> Arc<str> {
            Arc::from(self)
        }
    }

    macro_rules! deref_impl {
        (($(({$($life: lifetime)?}, $T: ty)),* $(,)?)) => {$(
            impl$(<$life>)? IntoTransfer for $T {
                fn into(self) -> Arc<str> {
                    <&$T as IntoTransfer>::into(&self)
                }
            }

            impl$(<$life>)? IntoTransfer for &$T {
                fn into(self) -> Arc<str> {
                    <&str as IntoTransfer>::into(&self)
                }
            }
        )*};
        (($($T: ty),*) {and} $(({$life: lifetime}, $life_ty: ty))*) => {
            deref_impl! {
                ($(({$life}, $life_ty))*, $(({}, $T)),*)
            }
        };
    }

    deref_impl! {
        (String, Box<str>, Rc<str>) {and} ({'a}, Cow<'a, str>)
    }

    impl IntoTransfer for Arc<str> {
        fn into(self) -> Arc<str> {
            self
        }
    }

    impl IntoTransfer for &Arc<str> {
        fn into(self) -> Arc<str> {
            Arc::clone(self)
        }
    }
}

/// trait for turning a string into a cheaply cloneable and transferable String
pub trait IntoTransfer: sealed::IntoTransfer {}

impl<T: sealed::IntoTransfer> IntoTransfer for T {}
