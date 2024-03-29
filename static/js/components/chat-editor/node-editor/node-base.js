export class NodeBase extends HTMLElement {
    #inputPin = null;
    #outputPins = [];

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

    sendModelUpdate() {
        this.dispatchEvent(new Event(".modelUpdate"));
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
        this.sendModelUpdate();
    }

    bindInput(element, lens) {
        element.textContent = lens.get();
        element.addEventListener("input", () => {
            lens.set(element.textContent);
            this.sendModelUpdate();
        });
    }
}

customElements.define("th-chat-editor-node-base", NodeBase);
