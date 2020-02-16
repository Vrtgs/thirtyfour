use yew::{html, Component, ComponentLink, Html, ShouldRender};

pub struct DropdownComponent {
    link: ComponentLink<Self>,
    current: String,
    label: String,
}

pub enum DropdownMsg {
    SelectOption1,
    SelectOption2,
}

impl Component for DropdownComponent {
    type Message = DropdownMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        DropdownComponent {
            link,
            current: String::from("Select"),
            label: String::from("None"),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            DropdownMsg::SelectOption1 => {
                self.current = String::from("Option 1");
                self.label = String::from("Option 1 selected");
                true // Indicate that the Component should re-render
            }
            DropdownMsg::SelectOption2 => {
                self.current = String::from("Option 2");
                self.label = String::from("Option 2 selected");
                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="section pure-u-1" data-section="section-dropdown">
                <div class="pure-u-1-1">
                    <b>{"Dropdown"}</b>
                    <br /><br />
                </div>
                <div class="pure-u-1-6">
                    <div class="pure-menu pure-menu-horizontal">
                        <ul class="pure-menu-list" id="dropdown-outer">
                            <li class="pure-menu-item pure-menu-has-children pure-menu-allow-hover" id="dropdown-inner">
                                <a href="#" id="menuLink1" class="pure-menu-link">{&self.current}</a>
                                <ul class="pure-menu-children">
                                    <li class="pure-menu-item">
                                        <a href="#" class="pure-menu-link"
                                            id="option1"
                                            onclick={self.link.callback(|_| DropdownMsg::SelectOption1)}>
                                            {"Option 1"}
                                        </a>
                                    </li>
                                    <li class="pure-menu-item">
                                        <a href="#" class="pure-menu-link"
                                            id="option2"
                                            onclick={self.link.callback(|_| DropdownMsg::SelectOption2)}>
                                            {"Option 2"}
                                        </a>
                                    </li>
                                </ul>
                            </li>
                        </ul>
                    </div>
                </div>
                <div class="pure-u-2-3 label" id="dropdown-result">
                    {&self.label}
                </div>
            </div>
        }
    }
}
