import { Chat, PlaybackError, getFactValue, setFactValue } from "../chat.js";
import * as nodes from "./node-editor/nodes.js";

export class Preview extends HTMLElement {
    static uniqueIDCounter = 0;

    constructor(model) {
        super();
        this.model = model;
        this.id = `th-chat-preview.${Preview.uniqueIDCounter++}`;
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

        this.facts = this.scrollDiv.appendChild(document.createElement("div"));
        this.facts.classList.add("facts");

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

        this.updateFacts();
    }

    updateFacts() {
        let factSet = new Set();

        for (let name in this.model.nodes) {
            let node = this.model.nodes[name];
            let referencedFacts = nodes.schema[node.kind].getFactReferences(node);
            referencedFacts.forEach((fact) => factSet.add(fact));
        }

        let factArray = Array.from(factSet).sort();

        this.facts.replaceChildren();
        for (let factName of factArray) {
            let fact = this.facts.appendChild(document.createElement("div"));
            fact.classList.add("fact");

            let checkbox = fact.appendChild(document.createElement("input"));
            checkbox.id = `${this.id}.fact.${factName}`;
            checkbox.type = "checkbox";
            checkbox.checked = !!getFactValue(factName);
            checkbox.addEventListener("change", () => {
                setFactValue(factName, checkbox.checked);
                this.updateFromModel();
            });

            let label = fact.appendChild(document.createElement("label"));
            label.textContent = factName;
            label.htmlFor = checkbox.id;
        }
    }
}

customElements.define("th-chat-preview", Preview);
