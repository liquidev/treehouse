import { NodeBase } from "./node-base.js";
import { Pin } from "./pin.js";
import * as lens from "treehouse/common/lens.js";

export class NodeStart extends NodeBase {
    connectedCallback() {
        super.connectedCallback();

        this.label = this.appendChild(document.createElement("p"));
        this.label.classList.add("label");
        this.label.textContent = "start";

        this.outputPin = this.appendChild(
            new Pin(this.modelNode, "output", lens.field(this.modelNode, "then"))
        );
        this.addOutputPin(this.outputPin);
    }
}

customElements.define("th-chat-editor-node-start", NodeStart);
