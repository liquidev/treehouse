export class Pin extends HTMLElement {
    static #id = 0;

    constructor(name, direction, value) {
        super();
        this.id = Pin.#id++;
        this.name = name;
        this.direction = direction;
        this.value = value;
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
