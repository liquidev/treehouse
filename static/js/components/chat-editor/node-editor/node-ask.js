import { NodeBase } from "./node-base.js";
import { Pin } from "./pin.js";
import * as lens from "treehouse/common/lens.js";

export class NodeAsk extends NodeBase {
    connectedCallback() {
        super.connectedCallback();

        this.updateFromModel();
    }

    updateFromModel() {
        super.updateFromModel();

        this.replaceChildren();

        this.setInputPin(this.appendChild(new Pin(this.modelNode, "input")));

        this.questions = this.appendChild(document.createElement("div"));
        this.questions.classList.add("questions");

        this.add = this.questions.appendChild(document.createElement("div"));
        this.add.classList.add("icon-button", "add");
        this.add.addEventListener("click", () => {
            this.modelNode.questions.push(this.newQuestion());
            this.updateFromModel();
            this.sendModelUpdate();
        });

        for (let i = 0; i < this.modelNode.questions.length; ++i) {
            let question = this.modelNode.questions[i];

            let questionContainer = this.questions.appendChild(document.createElement("div"));
            questionContainer.classList.add("question");

            let remove = questionContainer.appendChild(document.createElement("div"));
            remove.classList.add("icon-button", "remove");
            remove.addEventListener("click", () => {
                this.modelNode.questions.splice(i, 1);
                this.updateFromModel();
                this.sendModelUpdate();
            });

            let questionContent = questionContainer.appendChild(document.createElement("p"));
            questionContent.contentEditable = true;
            this.bindInput(questionContent, lens.field(question, "content"));

            this.addOutputPin(
                questionContainer.appendChild(
                    new Pin(this.modelNode, "output", lens.field(question, "then"))
                )
            );
        }

        // Elements are added progressively, which can cause the size of the node to change as it's
        // being built. Therefore the rendering cache has to be updated once it's entirely ready.
        this.updateRenderingCache();
    }

    newQuestion() {
        return { content: `Question ${this.modelNode.questions.length + 1}`, then: null };
    }
}

customElements.define("th-chat-editor-node-ask", NodeAsk);
