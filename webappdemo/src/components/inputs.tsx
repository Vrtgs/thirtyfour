import React, {useState} from "react";

export function Inputs(): JSX.Element {
    const [value, setValue] = useState("");
    const [label, setLabel] = useState("");

    return (
        <div className="section pure-u-1" data-section="section-input">
            <div className="pure-u-1-1">
                <b>Text Input</b>
                <br/><br/>
            </div>
            <div className="pure-u-1-6">
                <input type="text" name="input1"
                       onChange={(e) => {
                           let target = e.target as HTMLInputElement;
                           setValue(target.value);
                       }}
                       placeholder="Type text here"
                       size={15}
                       maxLength={20}
                       value={value} />

            </div>
            <div className="pure-u-1-6">
                <input type="text" name="input2"
                       checked={true}
                       onChange={(e) => {
                           let target = e.target as HTMLInputElement;
                           setValue(target.value);
                       }}
                       size={15}
                       maxLength={20}
                       value={value} />
            </div>
            <div className="pure-u-1-6">
                <button className="pure-button" id="button-set"
                        onClick={() => setLabel(value)}>
                    SET VALUE
                </button>
            </div>
            <div className="pure-u-1-3 label" id="input-result">
                {label}
            </div>
        </div>
    );
}
