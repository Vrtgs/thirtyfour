import React, {useState} from "react";

export function Buttons(): JSX.Element {
    const [label, setLabel] = useState("None");

    return (
        <div className="section pure-u-1" data-section="section-buttons">
        <div className="pure-u-1-1">
            <b>Buttons</b>
            <br/><br/>
        </div>
        <div className="pure-u-1-6">
            <button className="pure-button pure-button-primary" id="button1"
                    onMouseDown={() => setLabel("Button 1 down")}
                    onClick={() => setLabel("Button 1 clicked")}
                    onContextMenu={() => setLabel("Button 1 right-clicked")}
                    onDoubleClick={() => setLabel("Button 1 double-clicked")}>
                BUTTON 1
            </button>
        </div>
        <div className="pure-u-1-6">
            <button className="pure-button" id="button2"
                    onMouseDown={() => setLabel("Button 2 down")}
                    onClick={() => setLabel("Button 2 clicked")}
                    onContextMenu={() => setLabel("Button 2 right-clicked")}
                    onDoubleClick={() => setLabel("Button 2 double-clicked")}>
                BUTTON 2
            </button>
        </div>
        <div className="pure-u-2-3 label" id="button-result">
            {label}
        </div>
    </div>
    );
}
