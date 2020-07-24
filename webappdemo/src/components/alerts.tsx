import React, {useState} from "react";

export function Alerts(): JSX.Element {
    const [label, setLabel] = useState("");

    return (
        <div className="section pure-u-1" data-section="section-alerts">
            <div className="pure-u-1-1">
                <b>Alerts</b>
                <br/><br/>
            </div>
            <div className="pure-u-1-6">
                <button className="pure-button pure-button-primary" id="alertbutton1"
                        onClick={() => {
                            alert("Alert 1 showing");
                            setLabel("Alert 1 clicked");
                        }}>
                    ALERT 1
                </button>
            </div>
            <div className="pure-u-1-6">
                <button className="pure-button" id="alertbutton2"
                        onClick={() => {
                            let value = window.confirm("Alert 2 showing");
                            if (value) {
                                setLabel("Alert 2 clicked true");
                            } else {
                                setLabel("Alert 2 clicked false");
                            }
                        }}>
                    ALERT 2
                </button>
            </div>
            <div className="pure-u-1-6">
                <button className="pure-button" id="alertbutton3"
                        onClick={() => {
                            let value = prompt("Alert 3 showing");
                            if (value) {
                                setLabel(value);
                            } else {
                                setLabel("");
                            }
                        }}>
                    ALERT 3
                </button>
            </div>
            <div className="pure-u-1-3 label" id="alert-result">
                {label}
            </div>
        </div>
    );
}
