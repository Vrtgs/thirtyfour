use yew::{html, Component, ComponentLink, Html, InputData, ShouldRender};

pub struct InputComponent {
    link: ComponentLink<Self>,
    label: String,
    value: String,
}

pub enum InputMsg {
    GotInput(InputData),
    Click,
}

impl Component for InputComponent {
    type Message = InputMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        InputComponent {
            link,
            label: String::new(),
            value: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            InputMsg::GotInput(e) => {
                self.value = e.value;
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
                    <input type="text" name="input1"
                        oninput={self.link.callback(|e| InputMsg::GotInput(e))}
                        placeholder="Type text here"
                        size="15"
                        maxlength="20">
                        {&self.value}
                    </input>
                </div>
                <div class="pure-u-1-6">
                    <input type="text" name="input2"
                        value="default input text"
                        checked=true
                        size="15"
                        maxlength="20">
                        {&self.value}
                    </input>
                </div>
                <div class="pure-u-1-6">
                    <button class="pure-button" id="button-set"
                        onclick={self.link.callback(|_| InputMsg::Click)}>
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
