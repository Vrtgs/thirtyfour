#![recursion_limit = "512"]

use yew::{html, Component, ComponentLink, Html, ShouldRender};

use crate::components::alerts::AlertComponent;
use crate::components::buttons::ButtonComponent;
use crate::components::dragdrop::DragDropComponent;
use crate::components::dropdown::DropdownComponent;
use crate::components::inputs::InputComponent;

pub mod components {
    pub mod alerts;
    pub mod buttons;
    pub mod dragdrop;
    pub mod dropdown;
    pub mod inputs;
}

enum Page {
    Buttons,
    Dropdown,
    TextInput,
    Alerts,
    DragDrop,
    IFrame,
}

struct App {
    link: ComponentLink<Self>,
    page: Page,
}

enum Msg {
    Buttons,
    Dropdown,
    TextInput,
    Alerts,
    DragDrop,
    IFrame,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link,
            page: Page::Buttons,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Buttons => {
                self.page = Page::Buttons;
                true
            }
            Msg::Dropdown => {
                self.page = Page::Dropdown;
                true
            }
            Msg::TextInput => {
                self.page = Page::TextInput;
                true
            }
            Msg::Alerts => {
                self.page = Page::Alerts;
                true
            }
            Msg::DragDrop => {
                self.page = Page::DragDrop;
                true
            }
            Msg::IFrame => {
                self.page = Page::IFrame;
                true
            }
        }
    }

    fn view(&self) -> Html {
        let p = match self.page {
            Page::Buttons => html! { <ButtonComponent /> },
            Page::Dropdown => html! { <DropdownComponent /> },
            Page::TextInput => html! { <InputComponent /> },
            Page::Alerts => html! { <AlertComponent /> },
            Page::DragDrop => html! { <DragDropComponent /> },
            Page::IFrame => html! {
                <iframe
                    src="/" width="100%" height="400px"
                    id="iframeid1" name="iframename1">
                </iframe>
            },
        };

        html! {
            <div class="pure-g">
                <div class="pure-u-1">
                    <p align="center"><h1>{"Demo Web App"}</h1></p>
                </div>
                <div class="pure-u-1">
                    <p align="center">
                        {"This is a small demo web app for testing the "}
                        <a href="https://github.com/stevepryde/thirtyfour">{"thirtyfour"}</a>
                        {" Selenium library for Rust."}<br /><br />
                        {"It is built using the "}<a href="https://yew.rs">{"Yew Framework"}</a>{"."}
                    </p>
                </div>
                <div class="section pure-u-1" data-section="section-sections">
                    <div class="pure-u-1-1">
                        <b>{"Navigation"}</b>
                        <br /><br />
                    </div>
                    <div class="pure-u-1-6">
                        <button class="pure-button" id="pagebuttons"
                            onclick={self.link.callback(|_| Msg::Buttons)}>
                            { "BUTTONS" }
                        </button>
                    </div>
                    <div class="pure-u-1-6">
                        <button class="pure-button" id="pagedropdown"
                            onclick={self.link.callback(|_| Msg::Dropdown)}>
                            { "DROPDOWN" }
                        </button>
                    </div>
                    <div class="pure-u-1-6">
                        <button class="pure-button" id="pagetextinput"
                            onclick={self.link.callback(|_| Msg::TextInput)}>
                            { "TEXTINPUT" }
                        </button>
                    </div>
                    <div class="pure-u-1-6">
                        <button class="pure-button" id="pagealerts"
                            onclick={self.link.callback(|_| Msg::Alerts)}>
                            { "ALERTS" }
                        </button>
                    </div>
                    <div class="pure-u-1-6">
                        <button class="pure-button" id="pagedragdrop"
                            onclick={self.link.callback(|_| Msg::DragDrop)}>
                            { "DRAG AND DROP" }
                        </button>
                    </div>
                    <div class="pure-u-1-6">
                        <button class="pure-button" id="pageiframe"
                            onclick={self.link.callback(|_| Msg::IFrame)}>
                            { "IFRAME" }
                        </button>
                    </div>
                </div>
                <br />
                {p}
            </div>

        }
    }
}

fn main() {
    yew::start_app::<App>();
}
