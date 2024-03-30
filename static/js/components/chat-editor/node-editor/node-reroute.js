import { NodeBase } from "./node-base.js";
import { Pin } from "./pin.js";
import * as lens from "treehouse/common/lens.js";

export class NodeReroute extends NodeBase {
    connectedCallback() {
        super.connectedCallback();

        this.setInputPin(this.appendChild(new Pin(this.modelNode, "input")));

        this.label = this.appendChild(document.createElement("p"));
        this.label.classList.add("handle");
        this.label.textContent = "⣿⣿⣿";

        this.outputPin = this.appendChild(
            new Pin(this.modelNode, "output", lens.field(this.modelNode, "then"))
        );
        this.addOutputPin(this.outputPin);
    }
}

customElements.define("th-chat-editor-node-reroute", NodeReroute);
