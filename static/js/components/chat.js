import { addSpell, spell } from "treehouse/spells.js";
import { Branch } from "treehouse/tree.js";
import { getCharacterPictureSrc } from "./chat/characters.js";

const persistenceKey = "treehouse.chats";
let persistentState = JSON.parse(localStorage.getItem(persistenceKey)) || {};

persistentState.log ??= {};
persistentState.facts ??= {};
savePersistentState();

function savePersistentState() {
    localStorage.setItem(persistenceKey, JSON.stringify(persistentState));
}

export class Chat extends HTMLElement {
    constructor(id, model) {
        super();
        this.id = id;
        this.model = model;
    }

    connectedCallback() {
        let startNode = null;
        for (let name in this.model.nodes) {
            if (this.model.nodes[name].kind == "start") {
                startNode = name;
                break;
            }
        }
        if (startNode == null) {
            this.append("Chat has no start node. Did you forget to add one in?");
            return;
        }

        this.state = new ChatState(this, this.model);
        this.state.onInteract = () => {
            persistentState.log[this.id] = this.state.log;
            savePersistentState();
        };
        this.state.onPause = (name) => {
            this.dispatchEvent(Object.assign(new Event(".pause"), { atNode: name }));
        };
        this.state.onError = (error) => {
            this.dispatchEvent(Object.assign(new Event(".playbackError"), { error }));
        };
        this.state.animate = false;
        this.state.exec(startNode);
        this.state.animate = true;

        let log = persistentState.log[this.id];
        if (log != null) {
            this.state.animate = false;
            try {
                this.state.replay(log);
            } catch (error) {
                throw new PlaybackError(`uncaught error while replaying log`, { cause: error });
            }
            this.state.animate = true;
        }
    }

    editLog(then) {
        then(this.state.log);
        persistentState.log[this.id] = this.state.log;
        savePersistentState();
    }
}

customElements.define("th-chat", Chat);

export class PlaybackError extends Error {}

class Said extends HTMLElement {
    constructor({ character, expression, animate }) {
        super();
        this.character = character;
        this.expression = expression ?? "neutral";
        this.doAnimate = animate;
    }

    connectedCallback() {
        if (this.character != null) {
            this.picture = this.appendChild(document.createElement("div"));
            this.picture.classList.add(
                "picture",
                `character-${this.character}`,
                `expression-${this.expression}`
            );

            if (this.doAnimate) {
                this.picture.style.animation =
                    "th-chat-appear var(--transition-duration) forwards ease-out";
            }
        }

        this.textBoxes = this.appendChild(document.createElement("div"));
        this.textBoxes.classList.add("text-boxes");
    }

    addTextBox(content) {
        let textBox = this.textBoxes.appendChild(document.createElement("div"));
        textBox.classList.add("text-box");

        let textContainer = textBox.appendChild(document.createElement("span"));
        textContainer.classList.add("text-container");
        textContainer.innerHTML = content;

        if (this.doAnimate) {
            textBox.textFullyVisible = false;

            textBox.style.animation = "th-chat-appear var(--transition-duration) forwards ease-out";

            let waiter = new Waiter();
            let beginLetterAnimation = Said.#animateLettersInNode(waiter, textContainer);

            textBox.addEventListener("animationend", async (event) => {
                if (event.animationName == "th-chat-appear") {
                    await beginLetterAnimation();
                    textBox.dispatchEvent(new Event(".textFullyVisible"));
                }
            });

            window.addEventListener("mousedown", () => {
                waiter.skip = true;
            });
        } else {
            textBox.textFullyVisible = true;
        }

        return textBox;
    }

    static #delayAfterLetter(letter) {
        switch (letter) {
            case ".":
            case "!":
            case "?":
            case "…":
                return 300;
            case ",":
                return 250;
            default:
                return 15;
        }
    }

    static #animateLettersInNode(waiter, node) {
        let display = node.style.display;
        node.style.display = "none";

        let beginAnimation = async () => {
            node.style.display = display;
            for (let child of node.childNodes) {
                if (child instanceof Text) {
                    let text = child.textContent;
                    let container = document.createElement("span");
                    container.classList.add("animated-text");
                    child.replaceWith(container);

                    // TODO: As of 2024-03-25, Intl.Segmenter is not available on all major browser
                    // versions (on Firefox, it is only available in Nightly). This means we are not
                    // able to do a more proper Unicode-aware version of this for now.
                    for (let i = 0; i < text.length; ++i) {
                        let c = text.substring(i, i + 1);
                        if (waiter.skip) {
                            c = text.substring(i);
                        }
                        let span = container.appendChild(document.createElement("span"));
                        span.classList.add("animated-letter");
                        span.textContent = c;
                        if (waiter.skip) {
                            break;
                        }
                        await waiter.wait(Said.#delayAfterLetter(c));
                    }
                } else {
                    await Said.#animateLettersInNode(waiter, child)();
                }
            }
        };

        return beginAnimation;
    }
}

class Waiter {
    skip = false;

    wait(ms) {
        if (this.skip) {
            return new Promise((resolve) => resolve());
        } else {
            return new Promise((resolve) => setTimeout(resolve, ms));
        }
    }
}

customElements.define("th-chat-said", Said);

class Asked extends HTMLElement {
    constructor({ content, alreadyAsked, animate, animationDelay }) {
        super();
        this.content = content;
        this.alreadyAsked = alreadyAsked;

        this.doAnimate = animate;
        this.animationDelay = animationDelay;
    }

    connectedCallback() {
        this.button = document.createElement("button");
        this.button.innerHTML = this.content;
        this.button.addEventListener("click", (_) => {
            this.dispatchEvent(new Event(".click"));
        });
        if (this.alreadyAsked) {
            this.button.classList.add("asked");
        }
        this.appendChild(this.button);

        if (this.doAnimate) {
            this.style.opacity = "0%";
            this.style.animation = `th-chat-appear var(--transition-duration) ${
                this.animationDelay * 0.1
            }s forwards ease-out`;
        }
    }

    interactionFinished() {
        this.button.disabled = true;
    }
}

customElements.define("th-chat-asked", Asked);

class ChatState {
    animate = true;
    log = [];
    results = {};
    wereAsked = new Set();

    onInteract = () => {};
    onPause = (_name) => {};
    onError = (error) => {
        throw error;
    };

    animate = true;

    #currentSaid = null;

    constructor(container, model) {
        this.container = container;
        this.model = model;
    }

    // General control

    replay(log) {
        for (let entry of log) {
            let interactionResult = this.interact(entry);
            if (interactionResult != ChatState.interactionOk) {
                return interactionResult;
            }
        }
        return ChatState.interactionOk;
    }

    exec(name, by) {
        let node = this.model.nodes[name];
        if (node == null) {
            this.onError(
                Object.assign(new PlaybackError(`encountered an unconnected node`), {
                    atNode: by,
                })
            );
            return;
        }

        this.onPause(name);

        let results = this.results[name];
        this.results[name] = this[node.kind](name, node, results);
    }

    // Implementations of nodes

    start(name, node) {
        this.exec(node.then, name);
    }

    say(name, node) {
        if (
            this.#currentSaid == null ||
            this.#currentSaid.character != node.character ||
            this.#currentSaid.expression != node.expression
        ) {
            this.#currentSaid = new Said({
                character: node.character,
                expression: node.expression,
                animate: this.animate,
            });
            this.container.appendChild(this.#currentSaid);
        }

        let textBox = this.#currentSaid.addTextBox(node.content);
        textBox.addEventListener(".textFullyVisible", (_) => this.exec(node.then, name));
        // Kind of a shitty hack that works around us not being able to use a connectedCallback for
        // non-custom elements. We'd want to dispatch the .textFullyVisible event only whenever the
        // text box is connected, but as far as I know that's impossible to do without registering
        // a custom element.
        if (textBox.textFullyVisible) {
            this.exec(node.then, name);
        }
        this.#scrollIntoView(textBox);
    }

    ask(name, node) {
        this.#currentSaid = null;

        let questions = [];
        for (let i_ = 0; i_ < node.questions.length; ++i_) {
            let i = i_; // closures my lovely
            let key = `${name}[${i}]`;

            let question = node.questions[i];
            let asked = new Asked({
                content: question.content,
                alreadyAsked: this.wereAsked.has(key),
                animate: this.animate,
                animationDelay: i,
            });
            asked.addEventListener(".click", (_) => {
                this.interact({
                    kind: "ask.choose",
                    name,
                    option: i,
                    key,
                });
            });
            this.container.appendChild(asked);
            this.#scrollIntoView(asked);
            questions[i] = asked;
        }
        return questions;
    }

    set(name, node) {
        persistentState.facts[node.fact] = true;
        savePersistentState();
        this.exec(node.then, name);
    }

    reroute(name, node) {
        this.exec(node.then, name);
    }

    end() {}

    // Persistent restorable interactions

    static interactionOk = "ok";
    static interactionFailed = "fail";

    interact(interaction) {
        let node = this.model.nodes[interaction.name];
        if (node == null) {
            return ChatState.interactionFailed;
        }

        this.log.push(interaction);
        this.onInteract();

        switch (interaction.kind) {
            case "ask.choose":
                {
                    if (this.wereAsked.has(interaction.key)) {
                        this.log.pop();
                    }
                    this.wereAsked.add(interaction.key);

                    let questions = this.results[interaction.name];
                    let question = node.questions[interaction.option];
                    let asked = questions[interaction.option];
                    asked.interactionFinished();
                    this.exec(question.then, interaction.name);
                    for (let q of questions) {
                        if (q != asked) {
                            q.parentNode.removeChild(q);
                        }
                    }
                }
                break;
        }

        return ChatState.interactionOk;
    }

    // Utilities

    #scrollIntoView(element) {
        if (this.animate) {
            element.scrollIntoView({
                behavior: "smooth",
                block: "start",
            });
        }
    }
}

addSpell(
    "chat",
    class {
        constructor(branch) {
            let id = spell(branch, Branch).namedID;
            let model = JSON.parse(spell(branch, Branch).branchContent.textContent);
            branch.replaceWith(new Chat(id, model));
        }
    }
);
