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
    constructor({ content, character, expression, animate }) {
        super();
        this.content = content;
        this.character = character;
        this.expression = expression ?? "neutral";
        this.doAnimate = animate;
    }

    connectedCallback() {
        if (this.character != null) {
            this.picture = new Image(64, 64);
            this.picture.src = getCharacterPictureSrc(this.character, this.expression);
            this.picture.classList.add("picture");
            this.appendChild(this.picture);
        }

        this.textContainer = document.createElement("span");
        this.textContainer.innerHTML = this.content;
        this.textContainer.classList.add("text-container");
        this.appendChild(this.textContainer);

        if (this.doAnimate) {
            this.style.animation = "th-chat-appear var(--transition-duration) forwards ease-out";
            let beginLetterAnimation = this.#animateLetters();
            this.addEventListener("animationend", async (event) => {
                if (event.animationName == "th-chat-appear") {
                    await beginLetterAnimation();
                    this.dispatchEvent(new Event(".textFullyVisible"));
                }
            });
        } else {
            this.dispatchEvent(new Event(".textFullyVisible"));
        }
    }

    #animateLetters() {
        return Said.#animateLettersInNode(this.textContainer);
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

    static #animateLettersInNode(node) {
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
                    // versions (it is available on Nightly Firefox). This means we are not able to
                    // do a more proper Unicode-aware version of this for now.
                    for (let i = 0; i < text.length; ++i) {
                        let c = text.substring(i, i + 1);
                        let span = document.createElement("span");
                        span.classList.add("animated-letter");
                        span.textContent = c;
                        container.appendChild(span);
                        await wait(Said.#delayAfterLetter(c));
                    }
                } else {
                    await Said.#animateLettersInNode(child)();
                }
            }
        };

        return beginAnimation;
    }
}

function wait(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
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
    constructor(container, model) {
        this.container = container;
        this.model = model;
        this.log = [];
        this.results = {};
        this.wereAsked = new Set();
        this.onInteract = () => {};
        this.onPause = (_name) => {};
        this.onError = (error) => {
            throw error;
        };

        this.animate = true;
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
        let said = new Said({
            content: node.content,
            character: node.character,
            expression: node.expression,
            animate: this.animate,
        });
        said.addEventListener(".textFullyVisible", (_) => this.exec(node.then, name));
        this.container.appendChild(said);
        this.#scrollIntoView(said);
    }

    ask(name, node) {
        let questions = [];
        for (let i_ = 0; i_ < node.questions.length; ++i_) {
            let i = i_;
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
