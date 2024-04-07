import { NodeBase } from "./node-base.js";
import { Pin } from "./pin.js";
import * as lens from "treehouse/common/lens.js";

export class NodeCheck extends NodeBase {
    connectedCallback() {
        super.connectedCallback();

        this.setInputPin(this.appendChild(new Pin(this.modelNode, "input")));

        this.rows = this.appendChild(document.createElement("div"));
        this.rows.classList.add("rows");

        this.topRow = this.rows.appendChild(document.createElement("div"));
        this.topRow.classList.add("top-row");

        this.fact = this.topRow.appendChild(document.createElement("p"));
        this.fact.classList.add("fact");
        this.fact.contentEditable = true;
        this.bindInput(this.fact, lens.field(this.modelNode, "fact"));

        this.label = this.topRow.appendChild(document.createElement("p"));
        this.label.classList.add("label");
        this.label.textContent = " is set?";

        this.ifSetRow = this.rows.appendChild(document.createElement("div"));
        this.ifSetRow.classList.add("branch-row");

        this.ifSetRowLabel = this.ifSetRow.appendChild(document.createElement("p"));
        this.ifSetRowLabel.textContent = "yeah";

        this.ifSetPin = this.ifSetRow.appendChild(
            new Pin(this.modelNode, "output", lens.field(this.modelNode, "ifSetThen"))
        );
        this.addOutputPin(this.ifSetPin);

        this.ifNotSetRow = this.rows.appendChild(document.createElement("div"));
        this.ifNotSetRow.classList.add("branch-row");

        this.ifNotSetRowLabel = this.ifNotSetRow.appendChild(document.createElement("p"));
        this.ifNotSetRowLabel.textContent = "nope";

        this.ifNotSetPin = this.ifNotSetRow.appendChild(
            new Pin(this.modelNode, "output", lens.field(this.modelNode, "ifNotSetThen"))
        );
        this.addOutputPin(this.ifNotSetPin);
    }
}

customElements.define("th-chat-editor-node-check", NodeCheck);
