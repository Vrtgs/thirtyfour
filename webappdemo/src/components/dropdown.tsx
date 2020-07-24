import React, {useState} from "react";

export function DropDowns(): JSX.Element {
    const [current, setCurrent] = useState("Option 1");
    const [label, setLabel] = useState("");

    return (
        <div className="section pure-u-1" data-section="section-dropdown">
            <div className="pure-u-1-1">
                <b>Dropdown</b>
                <br/><br/>
            </div>
            <div className="pure-u-1-6">
                <div className="pure-menu pure-menu-horizontal">
                    <ul className="pure-menu-list" id="dropdown-outer">
                        <li className="pure-menu-item pure-menu-has-children pure-menu-allow-hover" id="dropdown-inner">
                            <a href="#/" id="menuLink1" className="pure-menu-link">{current}</a>
                            <ul className="pure-menu-children">
                                <li className="pure-menu-item">
                                    <a href="#/" className="pure-menu-link"
                                       id="option1"
                                       onClick={() => {
                                           setCurrent("Option 1");
                                           setLabel("Option 1 selected");
                                       }}>
                                        Option 1
                                    </a>
                                </li>
                                <li className="pure-menu-item">
                                    <a href="#/" className="pure-menu-link"
                                       id="option2"
                                       onClick={() => {
                                           setCurrent("Option 2");
                                           setLabel("Option 2 selected");
                                       }}>
                                        Option 2
                                    </a>
                                </li>
                            </ul>
                        </li>
                    </ul>
                </div>
            </div>
            <div className="pure-u-2-3 label" id="dropdown-result">
                {label}
            </div>
        </div>
    );
}
