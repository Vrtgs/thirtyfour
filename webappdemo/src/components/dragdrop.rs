use stdweb::web::event::{IDragEvent, IEvent};
use yew::{
    html, Component, ComponentLink, DragDropEvent, DragOverEvent, DragStartEvent, Html,
    ShouldRender,
};

pub struct DragDropComponent {
    label: String,
    link: ComponentLink<DragDropComponent>,
}

pub enum DragDropMsg {
    DragItem1(DragStartEvent),
    DragItem2(DragStartEvent),
    DropTarget(DragDropEvent),
    DragOver(DragOverEvent),
}

impl Component for DragDropComponent {
    type Message = DragDropMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        DragDropComponent {
            label: String::from("None"),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            DragDropMsg::DragItem1(e) => {
                self.label = String::from("Dragging ITEM 1");
                if let Some(x) = e.data_transfer() {
                    x.set_data("text", "ITEM 1");
                }

                true
            }
            DragDropMsg::DragItem2(e) => {
                self.label = String::from("Dragging ITEM 2");
                if let Some(x) = e.data_transfer() {
                    x.set_data("text", "ITEM 2");
                }
                true
            }
            DragDropMsg::DropTarget(e) => {
                e.prevent_default();
                if let Some(x) = e.data_transfer() {
                    let text = x.get_data("text");
                    self.label = format!("Dropped {}", text);
                }
                true
            }
            DragDropMsg::DragOver(e) => {
                e.prevent_default();
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
                    <div class="dragitem"
                        id="dragitem1"
                        draggable={true}
                        ondragstart={self.link.callback(|e| DragDropMsg::DragItem1(e))}>
                        { "ITEM 1" }
                    </div>
                </div>
                <div class="pure-u-1-6">
                    <div class="dragitem"
                        id="dragitem2"
                        draggable={true}
                        ondragstart={self.link.callback(|e| DragDropMsg::DragItem2(e))}>
                        { "ITEM 2" }
                    </div>
                </div>
                <div class="pure-u-2-3">
                    <div class="droptarget" id="dragdrop-result"
                        ondrop={self.link.callback(|e| DragDropMsg::DropTarget(e))}
                        ondragover={self.link.callback(|e| DragDropMsg::DragOver(e))}>
                    {&self.label}
                    </div>
                </div>
            </div>
        }
    }
}
