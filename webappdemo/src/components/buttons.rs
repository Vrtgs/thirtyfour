use stdweb::web::event::IEvent;
use yew::{html, Component, ComponentLink, ContextMenuEvent, Html, ShouldRender};

pub struct ButtonComponent {
    link: ComponentLink<Self>,
    label: String,
}

pub enum ButtonMsg {
    ButtonDown1,
    ButtonDown2,
    ClickButton1,
    ClickButton2,
    ContextClickButton1(ContextMenuEvent),
    ContextClickButton2(ContextMenuEvent),
    DoubleClickButton1,
    DoubleClickButton2,
}

impl Component for ButtonComponent {
    type Message = ButtonMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        ButtonComponent {
            link,
            label: String::from("None"),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ButtonMsg::ButtonDown1 => {
                self.label = String::from("Button 1 down");
                true
            }
            ButtonMsg::ButtonDown2 => {
                self.label = String::from("Button 2 down");
                true
            }
            ButtonMsg::ClickButton1 => {
                self.label = String::from("Button 1 clicked");
                true
            }
            ButtonMsg::ClickButton2 => {
                self.label = String::from("Button 2 clicked");
                true
            }
            ButtonMsg::ContextClickButton1(e) => {
                e.prevent_default();
                e.stop_propagation();
                self.label = String::from("Button 1 right-clicked");
                true
            }
            ButtonMsg::ContextClickButton2(e) => {
                e.prevent_default();
                e.stop_propagation();
                self.label = String::from("Button 2 right-clicked");
                true
            }
            ButtonMsg::DoubleClickButton1 => {
                self.label = String::from("Button 1 double-clicked");
                true
            }
            ButtonMsg::DoubleClickButton2 => {
                self.label = String::from("Button 2 double-clicked");
                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="section pure-u-1" data-section="section-buttons">
                <div class="pure-u-1-1">
                    <b>{"Buttons"}</b>
                    <br /><br />
                </div>
                <div class="pure-u-1-6">
                    <button class="pure-button pure-button-primary" id="button1"
                        onmousedown={self.link.callback(|_| ButtonMsg::ButtonDown1)}
                        onclick={self.link.callback(|_| ButtonMsg::ClickButton1)}
                        oncontextmenu={self.link.callback(|e| ButtonMsg::ContextClickButton1(e))}
                        ondoubleclick={self.link.callback(|_| ButtonMsg::DoubleClickButton1)}>
                        { "BUTTON 1" }
                    </button>
                </div>
                <div class="pure-u-1-6">
                    <button class="pure-button" id="button2"
                        onmousedown={self.link.callback(|_| ButtonMsg::ButtonDown2)}
                        onclick={self.link.callback(|_| ButtonMsg::ClickButton2)}
                        oncontextmenu={self.link.callback(|e| ButtonMsg::ContextClickButton2(e))}
                        ondoubleclick={self.link.callback(|_| ButtonMsg::DoubleClickButton2)}>
                        { "BUTTON 2" }
                    </button>
                </div>
                <div class="pure-u-2-3 label" id="button-result">
                    {&self.label}
                </div>
            </div>
        }
    }
}
