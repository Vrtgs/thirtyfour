import React, {useState} from "react";

export function DragDrop(): JSX.Element {
    const [label, setLabel] = useState("");

    return (
        <div className="section pure-u-1" data-section="section-dragdrop">
            <div className="pure-u-1-1">
                <b>Drag and drop</b>
                <br/><br/>
            </div>
            <div className="pure-u-1-6">
                <div className="dragitem"
                     id="dragitem1"
                     draggable={true}
                     onDragStart={(e) => {
                         setLabel("Dragging ITEM 1");
                         e.dataTransfer.setData("text", "ITEM 1");
                     }}>
                    ITEM 1
                </div>
            </div>
            <div className="pure-u-1-6">
                <div className="dragitem"
                     id="dragitem2"
                     draggable={true}
                     onDragStart={(e) => {
                         setLabel("Dragging ITEM 2");
                         e.dataTransfer.setData("text", "ITEM 2");
                     }}>
                    ITEM 2
                </div>
            </div>
            <div className="pure-u-2-3">
                <div className="droptarget" id="dragdrop-result"
                     onDrop={(e) => {
                         e.preventDefault();
                         let text = e.dataTransfer.getData("text");
                         if (text) {
                             setLabel(`Dropped ${text}`);
                         }
                     }}
                     onDragOver={(e) => {
                         e.preventDefault();
                         return true;
                     }}>
                    {label}
                </div>
            </div>
        </div>
    );
}
