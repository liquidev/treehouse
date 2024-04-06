import { getCharacterPictureSrc } from "treehouse/components/chat/characters.js";
import { NodeBase } from "./node-base.js";
import { Pin } from "./pin.js";
import * as lens from "treehouse/common/lens.js";
import { ContextMenu, contextMenus } from "../context-menu.js";

export class NodeSay extends NodeBase {
    connectedCallback() {
        super.connectedCallback();

        this.setInputPin(this.appendChild(new Pin(this.modelNode, "input")));

        this.details = this.appendChild(document.createElement("div"));
        this.details.classList.add("details");

        this.picture = this.details.appendChild(new Image(64, 64));
        this.picture.classList.add("picture");
        this.#updatePicture();
        this.picture.addEventListener("click", () => {
            let picker = new CharacterPicker();
            picker.addEventListener(".characterPicked", (event) => {
                this.modelNode.character = event.character;
                this.modelNode.expression = event.expression;
                this.#updatePicture();
                this.sendModelUpdate();
            });
            contextMenus.openAtDropdown(picker, this.picture);
        });

        this.content = this.details.appendChild(document.createElement("p"));
        this.content.classList.add("content");
        this.content.contentEditable = true;
        this.bindInput(this.content, lens.field(this.modelNode, "content"));

        this.outputPin = this.appendChild(
            new Pin(this.modelNode, "output", lens.field(this.modelNode, "then"))
        );
        this.addOutputPin(this.outputPin);
    }

    #updatePicture() {
        this.picture.src = getCharacterPictureSrc(
            this.modelNode.character,
            this.modelNode.expression
        );
    }
}

customElements.define("th-chat-editor-node-say", NodeSay);

class CharacterPicker extends ContextMenu {
    static characters = [
        { name: "coco", prettyName: "Coco", expressions: ["neutral", "eyes_closed"] },
        { name: "vick", prettyName: "Vick", expressions: ["neutral"] },
    ];

    connectedCallback() {
        super.connectedCallback();

        for (let character of CharacterPicker.characters) {
            let characterName = this.appendChild(document.createElement("p"));
            characterName.classList.add("character-name");
            characterName.textContent = character.prettyName;

            let expressions = this.appendChild(document.createElement("div"));
            expressions.classList.add("expressions");
            for (let expression of character.expressions) {
                let button = expressions.appendChild(document.createElement("div"));
                button.classList.add("button");
                button.addEventListener("click", () => {
                    this.dispatchEvent(
                        Object.assign(new Event(".characterPicked"), {
                            character: character.name,
                            expression,
                        })
                    );
                    this.close();
                });

                let picture = button.appendChild(new Image(64, 64));
                picture.src = getCharacterPictureSrc(character.name, expression);

                let label = button.appendChild(document.createElement("p"));
                label.classList.add("label");
                label.textContent = expression;
            }
        }
    }
}

customElements.define("th-chat-editor-node-say-character-picker", CharacterPicker);
