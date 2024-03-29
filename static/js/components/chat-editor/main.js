import { NodeEditor } from "./node-editor.js";
import { ModelViewer } from "./model-viewer.js";

class ChatEditor extends HTMLElement {
    async connectedCallback() {
        this.model = { nodes: {} };

        this.nodeEditor = this.appendChild(new NodeEditor(this.model));
        this.modelViewer = this.appendChild(new ModelViewer(this.model));

        this.nodeEditor.addEventListener(".modelUpdate", () => {
            this.modelViewer.updateFromModel();
        });
    }

    useModel(model) {
        this.model.nodes = model.nodes;
    }

    updateFromModel() {
        this.nodeEditor.updateFromModel();
        this.modelViewer.updateFromModel();
    }
}

customElements.define("th-chat-editor", ChatEditor);
