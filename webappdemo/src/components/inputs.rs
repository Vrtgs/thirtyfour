use yew::{html, Callback, ClickEvent, Component, ComponentLink, Html, InputData, ShouldRender};

pub struct InputComponent {
    label: String,
    value: String,
    oninput: Callback<InputData>,
    onclick: Callback<ClickEvent>,
}

pub enum InputMsg {
    GotInput(String),
    Click,
}

impl Component for InputComponent {
    type Message = InputMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        InputComponent {
            label: String::new(),
            value: String::new(),
            oninput: link.callback(|e: InputData| InputMsg::GotInput(e.value)),
            onclick: link.callback(|_| InputMsg::Click),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            InputMsg::GotInput(new_value) => {
                self.value = new_value;
                true // Indicate that the Component should re-render
            }
            InputMsg::Click => {
                self.label = self.value.clone();
                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="section pure-u-1" data-section="section-input" >
                <div class="pure-u-1-1">
                    <b>{"Text Input"}</b>
                    <br /><br />
                </div>
                <div class="pure-u-1-6">
                    <input type="text" name="input1" oninput=&self.oninput placeholder="Type text here" size="15" maxlength="20">{&self.value}</input>
                </div>
                <div class="pure-u-1-6">
                    <button class="pure-button" id="button-set"
                        onclick=&self.onclick>
                        { "SET VALUE" }
                    </button>
                </div>
                <div class="pure-u-1-3 label" name="input-result">
                    {&self.label}
                </div>
            </div>
        }
    }
}
