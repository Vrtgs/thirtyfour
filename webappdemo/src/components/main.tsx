import React, {useState} from "react";
import {Buttons} from "./buttons";
import {DragDrop} from "./dragdrop";
import {Alerts} from "./alerts";
import {DropDowns} from "./dropdown";
import {Inputs} from "./inputs";

enum Pages {
    Alerts,
    Buttons,
    DragDrop,
    DropDown,
    IFrame,
    Inputs
}

export function MainPage(): JSX.Element {
    const [page, setPage] = useState(Pages.Buttons);

    let curPage;
    switch(page) {
        case Pages.Alerts:
            curPage = <Alerts/>;
            break;
        case Pages.Buttons:
            curPage = <Buttons />;
            break;
        case Pages.DragDrop:
            curPage = <DragDrop />;
            break;
        case Pages.DropDown:
            curPage = <DropDowns />;
            break;
        case Pages.IFrame:
            curPage = (
                <iframe
                    src="/" width="100%" height="400px"
                    id="iframeid1" name="iframename1"
                    title={"iframe title"}>
                </iframe>
            );
            break;
        case Pages.Inputs:
            curPage = <Inputs />;
            break;
        default:
            curPage = <Buttons />;
    }


    return (
        <div className="pure-g">
            <div className="pure-u-1">
                <h1 style={{textAlign: "center"}}>Demo Web App</h1>
            </div>
            <div className="pure-u-1">
                <p style={{textAlign: "center"}}>
                    This is a small demo web app for testing the&nbsp;
                    <a href="https://github.com/stevepryde/thirtyfour">thirtyfour</a>
                    &nbsp;Selenium library for Rust.
                    <br/><br/>
                    It is built in React/Typescript for fast build times.
                </p>
            </div>
            <div className="section pure-u-1" data-section="section-sections">
                <div className="pure-u-1-1">
                    <b>Navigation</b>
                    <br/><br/>
                </div>
                <div className="pure-u-1-6">
                    <button className="pure-button" id="pagebuttons"
                            onClick={() => setPage(Pages.Buttons)}>
                        BUTTONS
                    </button>
                </div>
                <div className="pure-u-1-6">
                    <button className="pure-button" id="pagedropdown"
                            onClick={() => setPage(Pages.DropDown)}>
                        DROPDOWN
                    </button>
                </div>
                <div className="pure-u-1-6">
                    <button className="pure-button" id="pagetextinput"
                            onClick={() => setPage(Pages.Inputs)}>
                        TEXTINPUT
                    </button>
                </div>
                <div className="pure-u-1-6">
                    <button className="pure-button" id="pagealerts"
                            onClick={() => setPage(Pages.Alerts)}>
                        ALERTS
                    </button>
                </div>
                <div className="pure-u-1-6">
                    <button className="pure-button" id="pagedragdrop"
                            onClick={() => setPage(Pages.DragDrop)}>
                        DRAG AND DROP
                    </button>
                </div>
                <div className="pure-u-1-6">
                    <button className="pure-button" id="pageiframe"
                            onClick={() => setPage(Pages.IFrame)}>
                        IFRAME
                    </button>
                </div>
            </div>
            <br/>
            {curPage}
        </div>
    );
}
