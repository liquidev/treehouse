export class ModelViewer extends HTMLElement {
    constructor(model) {
        super();
        this.model = model;
    }

    connectedCallback() {
        this.textArea = this.appendChild(document.createElement("input"));
        this.textArea.type = "text";
        this.textArea.rows = 1;
        this.updateFromModel();
    }

    updateFromModel() {
        this.textArea.value = JSON.stringify(this.model);
    }
}

customElements.define("th-chat-model-viewer", ModelViewer);
