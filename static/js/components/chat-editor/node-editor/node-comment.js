import { NodeBase } from "./node-base.js";
import { Pin } from "./pin.js";
import * as lens from "treehouse/common/lens.js";

export class NodeComment extends NodeBase {
    connectedCallback() {
        super.connectedCallback();

        this.comment = this.appendChild(document.createElement("p"));
        this.comment.classList.add("comment");
        this.comment.contentEditable = true;
        this.bindInput(this.comment, lens.field(this.modelNode, "content"));
    }
}

customElements.define("th-chat-editor-node-comment", NodeComment);
