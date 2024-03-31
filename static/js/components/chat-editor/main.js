import { NodeEditor } from "./node-editor.js";
import { ModelViewer } from "./model-viewer.js";
import { Preview } from "./preview.js";

class ChatEditor extends HTMLElement {
    async connectedCallback() {
        this.model = { nodes: {} };

        this.verticalSplit = this.appendChild(document.createElement("div"));
        this.verticalSplit.classList.add("vertical-split", "preview-visible");

        this.nodeEditorArea = this.verticalSplit.appendChild(document.createElement("div"));
        this.nodeEditorArea.classList.add("node-editor-area");
        this.nodeEditor = this.nodeEditorArea.appendChild(new NodeEditor(this.model));

        this.preview = this.verticalSplit.appendChild(new Preview(this.model));
        this.preview.addEventListener(".pause", (event) => {
            this.nodeEditor.markNodeAsPaused(event.atNode);
        });
        this.preview.addEventListener(".playbackError", (event) => {
            this.nodeEditor.markNodeAsErrorSource(event.atNode);
        });

        this.previewToggle = this.nodeEditorArea.appendChild(document.createElement("div"));
        this.previewToggle.classList.add("preview-toggle");
        this.previewToggle.textContent = "Preview";
        this.previewToggle.addEventListener("click", () => {
            this.verticalSplit.classList.toggle("preview-visible");
        });

        this.modelViewer = this.appendChild(new ModelViewer(this.model));

        this.nodeEditor.addEventListener(".modelUpdate", () => {
            this.modelViewer.updateFromModel();
            this.preview.updateFromModel();
        });
    }

    useModel(model) {
        this.model.nodes = model.nodes;
    }

    updateFromModel() {
        this.nodeEditor.updateFromModel();
        this.modelViewer.updateFromModel();
        this.preview.updateFromModel();
    }
}

customElements.define("th-chat-editor", ChatEditor);
