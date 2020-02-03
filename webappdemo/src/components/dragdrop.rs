use stdweb::web::event::{IDragEvent, IEvent};
use yew::{
    html, Callback, Component, ComponentLink, DragDropEvent, DragOverEvent, DragStartEvent, Html,
    ShouldRender,
};

pub struct DragDropComponent {
    label: String,
    ondrag_button1: Callback<DragStartEvent>,
    ondrag_button2: Callback<DragStartEvent>,
    ondrop_target: Callback<DragDropEvent>,
    ondragover: Callback<DragOverEvent>,
}

pub enum DragDropMsg {
    DragButton1,
    DragButton2,
    DropTarget(String),
    DragOver(String),
}

impl Component for DragDropComponent {
    type Message = DragDropMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        DragDropComponent {
            label: String::from("None"),
            ondrag_button1: link.callback(|e: DragStartEvent| {
                if let Some(x) = e.data_transfer() {
                    x.set_data("text", "BUTTON 1");
                }

                DragDropMsg::DragButton1
            }),
            ondrag_button2: link.callback(|e: DragStartEvent| {
                if let Some(x) = e.data_transfer() {
                    x.set_data("text", "BUTTON 2");
                }
                DragDropMsg::DragButton2
            }),
            ondrop_target: link.callback(|e: DragDropEvent| {
                e.prevent_default();
                match e.data_transfer() {
                    Some(x) => {
                        let text = x.get_data("text");
                        DragDropMsg::DropTarget(text)
                    }
                    None => DragDropMsg::DropTarget(String::from("UNKNOWN")),
                }
            }),
            ondragover: link.callback(|e: DragOverEvent| {
                e.prevent_default();
                match e.data_transfer() {
                    Some(x) => {
                        let text = x.get_data("text");
                        DragDropMsg::DragOver(text)
                    }
                    None => DragDropMsg::DragOver(String::from("UNKNOWN")),
                }
            }),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            DragDropMsg::DragButton1 => {
                self.label = String::from("Button 1 dragging");
                true
            }
            DragDropMsg::DragButton2 => {
                self.label = String::from("Button 2 dragging");
                true
            }
            DragDropMsg::DropTarget(x) => {
                self.label = format!("Dropped {}", x);
                true
            }
            DragDropMsg::DragOver(x) => {
                self.label = format!("Dragging {}", x);
                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="section pure-u-1" data-section="section-dragdrop">
                <div class="pure-u-1-1">
                    <b>{"Drag and drop"}</b>
                    <br /><br />
                </div>
                <div class="pure-u-1-6">
                    <button class="pure-button pure-button-primary" id="buttondrag1"
                        draggable="true"
                        ondragstart=&self.ondrag_button1>
                        { "BUTTON 1" }
                    </button>
                </div>
                <div class="pure-u-1-6">
                    <button class="pure-button" id="buttondrag2"
                        draggable="true"
                        ondragstart=&self.ondrag_button2>
                        { "BUTTON 2" }
                    </button>
                </div>
                <div class="pure-u-2-3 label" id="dragdrop-result"
                    ondrop=&self.ondrop_target
                    ondragover=&self.ondragover>
                    {&self.label}
                </div>
            </div>
        }
    }
}
