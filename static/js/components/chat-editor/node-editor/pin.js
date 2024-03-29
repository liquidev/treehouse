import * as lens from "treehouse/common/lens.js";

export class Pin extends HTMLElement {
    static #id = 0;

    constructor(name, direction, value) {
        super();
        this.id = Pin.#id++;
        this.name = name;
        this.direction = direction;
        this.value = value;

        if (this.direction == "output") {
            this.#highlightNull(this.value.get());
            this.value = lens.listen(this.value, (l, newValue) => {
                this.#highlightNull(newValue);
            });
        }
    }

    connectedCallback() {
        this.classList.add(this.direction);
        this.classList.add("icon-button");

        this.addEventListener("mousedown", (event) => {
            if (event.button == 0) {
                event.preventDefault();
                event.stopPropagation();
                this.dispatchEvent(new Event(".beginDrag"));
            } else if (event.button == 2) {
                event.preventDefault();
                event.stopPropagation();
                this.dispatchEvent(new Event(".disconnect"));
            }
        });
    }

    beginConnecting() {
        this.classList.add("connecting");
    }

    endConnecting() {
        this.classList.remove("connecting");
    }

    #highlightNull(newValue) {
        if (newValue == null) {
            this.classList.add("dangling");
            this.title = "Unconnected output pin. The chat runtime will crash upon executing it.";
        } else {
            this.classList.remove("dangling");
        }
    }

    get connectionX() {
        if (this.direction == "output") {
            return this.offsetWidth;
        } else {
            return 0;
        }
    }

    get connectionY() {
        return this.offsetHeight / 2;
    }
}

customElements.define("th-chat-editor-node-pin", Pin);
