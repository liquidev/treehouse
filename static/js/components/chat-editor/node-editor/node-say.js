import { NodeBase } from "./node-base.js";
import { Pin } from "./pin.js";
import * as lens from "treehouse/common/lens.js";

export class NodeSay extends NodeBase {
    connectedCallback() {
        super.connectedCallback();

        this.setInputPin(this.appendChild(new Pin(this.modelNode, "input")));

        this.details = this.appendChild(document.createElement("div"));
        this.details.classList.add("details");

        this.character = this.details.appendChild(document.createElement("p"));
        this.character.classList.add("character");
        this.character.textContent = this.modelNode.character;

        this.content = this.details.appendChild(document.createElement("p"));
        this.content.classList.add("content");
        this.content.textContent = this.modelNode.content;
        this.content.contentEditable = true;

        this.outputPin = this.appendChild(
            new Pin(
                this.modelNode,
                "output",
                lens.field(this.modelNode, "then")
            )
        );
        this.addOutputPin(this.outputPin);
    }
}

customElements.define("th-chat-editor-node-say", NodeSay);
