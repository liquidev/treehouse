import { addSpell, spell } from "treehouse/spells.js";
import { Branch } from "treehouse/tree.js";

const persistenceKey = "treehouse.chats";
let persistentState = JSON.parse(localStorage.getItem(persistenceKey)) || {};

persistentState.log ??= {};
savePersistentState();

function savePersistentState() {
    localStorage.setItem(persistenceKey, JSON.stringify(persistentState));
}

class Chat extends HTMLElement {
    constructor(branch) {
        super();
        this.branch = branch;
    }

    connectedCallback() {
        this.id = spell(this.branch, Branch).namedID;
        this.model = JSON.parse(spell(this.branch, Branch).branchContent.textContent);

        this.state = new ChatState(this, this.model);
        this.state.onInteract = () => {
            persistentState.log[this.id] = this.state.log;
            savePersistentState();
        };
        this.state.exec("init");

        let log = persistentState.log[this.id];
        if (log != null) {
            this.state.replay(log);
        }
    }
}

customElements.define("th-chat", Chat);

class Said extends HTMLElement {
    constructor({ content }) {
        super();
        this.content = content;
    }

    connectedCallback() {
        this.innerHTML = this.content;
        this.dispatchEvent(new Event(".animationsComplete"));
    }
}

customElements.define("th-chat-said", Said);

class Asked extends HTMLElement {
    constructor({ content, alreadyAsked }) {
        super();
        this.content = content;
        this.alreadyAsked = alreadyAsked;
    }

    connectedCallback() {
        this.button = document.createElement("button");
        this.button.innerHTML = this.content;
        this.button.addEventListener("click", _ => {
            this.dispatchEvent(new Event(".click"));
        });
        if (this.alreadyAsked) {
            this.button.classList.add("asked");
        }
        this.appendChild(this.button);
    }

    interactionFinished() {
        this.button.disabled = true;
    }
}

customElements.define("th-chat-asked", Asked);

class ChatState {
    constructor(container, model) {
        this.container = container;
        this.model = model;
        this.log = [];
        this.results = {};
        this.wereAsked = new Set();
        this.onInteract = _ => {};
    }

    replay(log) {
        for (let entry of log) {
            this.interact(entry);
        }
    }

    exec(name) {
        let node = this.model.nodes[name];
        let results = this.results[name];
        this.results[name] = this[node.kind](name, node, results);
    }

    say(_, node) {
        let said = new Said({ content: node.content });
        said.addEventListener(".animationsComplete", _ => this.exec(node.then));
        this.container.appendChild(said);
    }

    ask(name, node) {
        let questions = [];
        for (let i_ = 0; i_ < node.questions.length; ++i_) {
            let i = i_;
            let key = `${name}[${i}]`;

            let question = node.questions[i];
            let asked = new Asked({ content: question.content, alreadyAsked: this.wereAsked.has(key) });
            asked.addEventListener(".click", _ => {
                this.interact({
                    kind: "ask.choose",
                    name,
                    option: i,
                    key,
                });
            });
            this.container.appendChild(asked);
            questions[i] = asked;
        }
        return questions;
    }

    end() {}

    interact(interaction) {
        let node = this.model.nodes[interaction.name];

        this.log.push(interaction);
        this.onInteract();

        switch (interaction.kind) {
            case "ask.choose": {
                if (this.wereAsked.has(interaction.key)) {
                    this.log.pop();
                }
                this.wereAsked.add(interaction.key);

                let questions = this.results[interaction.name];
                let question = node.questions[interaction.option];
                let asked = questions[interaction.option];
                asked.interactionFinished();
                this.exec(question.then);
                for (let q of questions) {
                    if (q != asked) {
                        q.parentNode.removeChild(q);
                    }
                }
            }
                break;
        }
    }
}

addSpell("chat", class {
    constructor(branch) {
        branch.replaceWith(new Chat(branch));
    }
});
