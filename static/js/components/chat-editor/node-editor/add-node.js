import { ContextMenu } from "../context-menu.js";

export class AddNode extends ContextMenu {
    static options = [
        { name: "say", description: "Make a character say something" },
        { name: "ask", description: "Give the player a dialog choice" },
        { name: "set", description: "Store a bit of information for later" },
        { name: "end", description: "End the conversation" },
    ];

    static templates = {
        say: {
            character: "coco",
            expression: "neutral",
            content: "Title T",
            then: null,
        },
        ask: {
            questions: [
                {
                    content: "Question 1",
                    then: null,
                },
            ],
        },
        set: {
            fact: "example/fact",
        },
        end: {},
    };

    connectedCallback() {
        super.connectedCallback();

        let title = this.appendChild(document.createElement("p"));
        title.classList.add("title");
        title.textContent = "Create a node";

        for (let option of AddNode.options) {
            let container = this.appendChild(document.createElement("div"));
            container.classList.add("option");

            let name = container.appendChild(document.createElement("p"));
            name.classList.add("name");
            name.textContent = option.name;

            let description = container.appendChild(document.createElement("p"));
            name.classList.add("description");
            description.textContent = option.description;

            container.addEventListener("click", (event) => {
                event.stopPropagation();

                let modelNode = structuredClone(AddNode.templates[option.name]);
                modelNode.kind = option.name;

                this.close();
                this.dispatchEvent(Object.assign(new Event(".addNode"), { modelNode }));
            });
        }
    }
}

customElements.define("th-chat-editor-add-node", AddNode);
