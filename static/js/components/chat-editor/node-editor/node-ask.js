import { NodeBase } from "./node-base.js";
import { Pin } from "./pin.js";
import * as lens from "treehouse/common/lens.js";

export class NodeAsk extends NodeBase {
    connectedCallback() {
        super.connectedCallback();

        this.setInputPin(this.appendChild(new Pin(this.modelNode, "input")));

        this.questions = this.appendChild(document.createElement("div"));
        this.questions.classList.add("questions");
        for (let question of this.modelNode.questions) {
            let questionContainer = this.questions.appendChild(
                document.createElement("div")
            );
            questionContainer.classList.add("question");

            let questionContent = questionContainer.appendChild(
                document.createElement("p")
            );
            questionContent.textContent = question.content;
            questionContent.contentEditable = true;

            this.addOutputPin(
                questionContainer.appendChild(
                    new Pin(
                        this.modelNode,
                        "output",
                        lens.field(question, "then")
                    )
                )
            );
        }
    }
}

customElements.define("th-chat-editor-node-ask", NodeAsk);
