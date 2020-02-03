use stdweb::web::event::IEvent;
use yew::{
    html, Callback, ClickEvent, Component, ComponentLink, ContextMenuEvent, DoubleClickEvent, Html,
    MouseDownEvent, ShouldRender,
};

pub struct ButtonComponent {
    label: String,
    ondown_button1: Callback<MouseDownEvent>,
    ondown_button2: Callback<MouseDownEvent>,
    onclick_button1: Callback<ClickEvent>,
    onclick_button2: Callback<ClickEvent>,
    oncontextclick_button1: Callback<ContextMenuEvent>,
    oncontextclick_button2: Callback<ContextMenuEvent>,
    ondoubleclick_button1: Callback<DoubleClickEvent>,
    ondoubleclick_button2: Callback<DoubleClickEvent>,
}

pub enum ButtonMsg {
    ButtonDown1,
    ButtonDown2,
    ClickButton1,
    ClickButton2,
    ContextClickButton1,
    ContextClickButton2,
    DoubleClickButton1,
    DoubleClickButton2,
}

impl Component for ButtonComponent {
    type Message = ButtonMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        ButtonComponent {
            label: String::from("None"),
            ondown_button1: link.callback(|_| ButtonMsg::ButtonDown1),
            ondown_button2: link.callback(|_| ButtonMsg::ButtonDown2),
            onclick_button1: link.callback(|_| ButtonMsg::ClickButton1),
            onclick_button2: link.callback(|_| ButtonMsg::ClickButton2),
            oncontextclick_button1: link.callback(|e: ContextMenuEvent| {
                e.prevent_default();
                e.stop_propagation();
                ButtonMsg::ContextClickButton1
            }),
            oncontextclick_button2: link.callback(|e: ContextMenuEvent| {
                e.prevent_default();
                e.stop_propagation();
                ButtonMsg::ContextClickButton2
            }),
            ondoubleclick_button1: link.callback(|_| ButtonMsg::DoubleClickButton1),
            ondoubleclick_button2: link.callback(|_| ButtonMsg::DoubleClickButton2),
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
            ButtonMsg::ContextClickButton1 => {
                self.label = String::from("Button 1 right-clicked");
                true
            }
            ButtonMsg::ContextClickButton2 => {
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
                        onmousedown=&self.ondown_button1
                        onclick=&self.onclick_button1
                        oncontextmenu=&self.oncontextclick_button1
                        ondoubleclick=&self.ondoubleclick_button1>
                        { "BUTTON 1" }
                    </button>
                </div>
                <div class="pure-u-1-6">
                    <button class="pure-button" id="button2"
                        onmousedown=&self.ondown_button2
                        onclick=&self.onclick_button2
                        oncontextmenu=&self.oncontextclick_button2
                        ondoubleclick=&self.ondoubleclick_button2>
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
