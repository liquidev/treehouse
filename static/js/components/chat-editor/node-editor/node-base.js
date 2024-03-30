import { getPositionRelativeToAncestor } from "./layout.js";

export class NodeBase extends HTMLElement {
    #inputPin = null;
    #outputPins = [];

    #pinRectCache = new Map();

    constructor(model, nodeName) {
        super();
        this.model = model;
        this.name = nodeName;
    }

    get modelNode() {
        return this.model.nodes[this.name];
    }

    get type() {
        return nodeTypes[this.modelNode.kind];
    }

    setInputPin(pin) {
        this.#inputPin = pin;
        this.#hookPin(pin);
    }

    addOutputPin(pin) {
        this.#outputPins.push(pin);
        this.#hookPin(pin);
    }

    #hookPin(pin) {
        pin.addEventListener(".beginDrag", () => {
            this.dispatchEvent(Object.assign(new Event(".pinDrag"), { pin }));
        });

        pin.addEventListener(".disconnect", () => {
            this.dispatchEvent(Object.assign(new Event(".pinDisconnect"), { pin }));
        });

        pin.addEventListener("mouseenter", () => {
            this.dispatchEvent(Object.assign(new Event(".pinHover"), { pin }));
        });
        pin.addEventListener("mouseleave", () => {
            this.dispatchEvent(Object.assign(new Event(".pinEndHover"), { pin }));
        });

        this.#updatePinRect(pin);
    }

    get inputPin() {
        if (this.#inputPin == null) {
            throw new Error("input pin was not set");
        }
        return this.#inputPin;
    }

    get outputPins() {
        return this.#outputPins;
    }

    connectedCallback() {
        this.classList.add("th-chat-editor-node");

        this.addEventListener("mousedown", (event) => {
            if (event.target == this && event.button == 0) {
                event.preventDefault();
                event.stopPropagation();

                document.activeElement.blur();
                this.focus();

                this.dispatchEvent(new Event(".select"));
            }
        });

        this.updateTransform();
    }

    #sendModelUpdateEvent() {
        this.dispatchEvent(new Event(".modelUpdate"));
    }

    sendModelUpdate() {
        this.#updatePinRects();
        this.#sendModelUpdateEvent();
    }

    updateFromModel() {
        this.#inputPin = null;
        this.#outputPins.splice(0);
    }

    updateTransform() {
        let [x, y] = this.modelNode.position;
        this.style.transform = `translate(${x}px, ${y}px)`;
    }

    move(deltaX, deltaY) {
        this.modelNode.position[0] += deltaX;
        this.modelNode.position[1] += deltaY;
        this.updateTransform();
        this.#sendModelUpdateEvent();
    }

    bindInput(element, lens) {
        element.textContent = lens.get();
        NodeBase.#updateEmptiness(element);
        element.addEventListener("input", () => {
            lens.set(element.textContent);
            NodeBase.#updateEmptiness(element);
            this.sendModelUpdate();
        });
    }

    static #updateEmptiness(element) {
        if (element.textContent.length == 0) {
            element.classList.add("empty");
            // Get rid of any leftover elements; Firefox leaves behind a <br>.
            element.textContent = "";
        } else {
            element.classList.remove("empty");
        }
    }

    #updatePinRects() {
        this.#pinRectCache.clear();
        if (this.#inputPin != null) {
            this.#updatePinRect(this.inputPin);
        }
        for (let pin of this.outputPins) {
            this.#updatePinRect(pin);
        }
    }

    #updatePinRect(pin) {
        let [x, y] = getPositionRelativeToAncestor(this, pin);
        let width = pin.offsetWidth;
        let height = pin.offsetHeight;
        this.#pinRectCache.set(pin, { x, y, width, height });
    }

    getPinRect(pin) {
        return this.#pinRectCache.get(pin);
    }

    updateRenderingCache() {
        this.#updatePinRects();
    }
}

customElements.define("th-chat-editor-node-base", NodeBase);
