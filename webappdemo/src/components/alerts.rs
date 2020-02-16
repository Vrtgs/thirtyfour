use stdweb::{js, Value};
use yew::{html, Component, ComponentLink, Html, ShouldRender};

pub fn show_alert(message: String) {
    js! { alert(@{message}); };
}

pub fn show_confirm(message: String) -> bool {
    if let Value::Bool(x) = js! { return confirm(@{message}); } {
        return x;
    }
    false
}

pub fn show_prompt(message: String, default: String) -> String {
    if let Value::String(x) = js! { return prompt(@{message}, @{default}); } {
        return x;
    }
    String::from("Cancelled")
}

pub struct AlertComponent {
    link: ComponentLink<Self>,
    label: String,
}

pub enum AlertMsg {
    ClickButton1,
    ClickButton2,
    ClickButton3,
}

impl Component for AlertComponent {
    type Message = AlertMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        AlertComponent {
            link,
            label: String::from("None"),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            AlertMsg::ClickButton1 => {
                self.label = String::from("Alert 1 clicked");
                show_alert(String::from("Alert 1 showing"));
                true // Indicate that the Component should re-render
            }
            AlertMsg::ClickButton2 => {
                let result = show_confirm(String::from("Alert 2 showing"));
                if result {
                    self.label = String::from("Alert 2 clicked true");
                } else {
                    self.label = String::from("Alert 2 clicked false");
                }
                true
            }
            AlertMsg::ClickButton3 => {
                self.label = show_prompt(String::from("Alert 3 showing"), String::from("yes"));
                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="section pure-u-1" data-section="section-alerts">
                <div class="pure-u-1-1">
                    <b>{"Alerts"}</b>
                    <br /><br />
                </div>
                <div class="pure-u-1-6">
                    <button class="pure-button pure-button-primary" id="alertbutton1"
                        onclick={self.link.callback(|_| AlertMsg::ClickButton1)}>
                        { "ALERT 1" }
                    </button>
                </div>
                <div class="pure-u-1-6">
                    <button class="pure-button" id="alertbutton2"
                        onclick={self.link.callback(|_| AlertMsg::ClickButton2)}>
                        { "ALERT 2" }
                    </button>
                </div>
                <div class="pure-u-1-6">
                    <button class="pure-button" id="alertbutton3"
                        onclick={self.link.callback(|_| AlertMsg::ClickButton3)}>
                        { "ALERT 3" }
                    </button>
                </div>
                <div class="pure-u-1-3 label" id="alert-result">
                    {&self.label}
                </div>
            </div>
        }
    }
}
