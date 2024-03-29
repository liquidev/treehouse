import { NodeBase } from "./node-base.js";
import { Pin } from "./pin.js";

export class NodeEnd extends NodeBase {
    connectedCallback() {
        super.connectedCallback();

        this.setInputPin(this.appendChild(new Pin(this.modelNode, "input")));

        this.label = this.appendChild(document.createElement("p"));
        this.label.classList.add("label");
        this.label.textContent = "end";
    }
}

customElements.define("th-chat-editor-node-end", NodeEnd);
