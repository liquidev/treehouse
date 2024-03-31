import { Chat, PlaybackError } from "../chat.js";

export class Preview extends HTMLElement {
    constructor(model) {
        super();
        this.model = model;
    }

    connectedCallback() {
        this.controls = this.appendChild(document.createElement("div"));
        this.controls.classList.add("controls");

        this.reset = this.controls.appendChild(document.createElement("button"));
        this.reset.textContent = "Reset";
        this.reset.addEventListener("click", () => {
            this.chat.editLog((log) => {
                log.splice(0);
            });
            this.updateFromModel();
        });

        this.stepBack = this.controls.appendChild(document.createElement("button"));
        this.stepBack.textContent = "Step back";
        this.stepBack.addEventListener("click", () => {
            this.chat.editLog((log) => {
                log.pop();
            });
            this.updateFromModel();
        });

        this.scrollDiv = this.appendChild(document.createElement("div"));
        this.scrollDiv.classList.add("scroll");

        this.chat = this.scrollDiv.appendChild(document.createElement("div")); // replaced later
        this.errors = this.scrollDiv.appendChild(document.createElement("p"));
        this.errors.classList.add("errors");

        this.updateFromModel();
    }

    updateFromModel() {
        let oldChat = this.chat;

        this.errors.textContent = "";

        this.chat = new Chat("chat-editor-preview", this.model);

        this.chat.addEventListener(".pause", (event) => {
            this.dispatchEvent(Object.assign(new Event(".pause"), { atNode: event.atNode }));
        });

        this.chat.addEventListener(".playbackError", (event) => {
            this.errors.textContent = `Error: ${event.error.message}`;

            if (event.error instanceof PlaybackError) {
                this.dispatchEvent(
                    Object.assign(new Event(".playbackError"), { atNode: event.error.atNode })
                );
                this.errors.textContent += `\n(look at the node graph to see where the error occured)`;
            }
        });

        oldChat.replaceWith(this.chat);
    }
}

customElements.define("th-chat-preview", Preview);
