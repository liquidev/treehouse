// A frameworking class assigning some CSS classes to the canvas to make it integrate nicer with CSS.
export class Frame extends HTMLCanvasElement {
    static fontFace = "RecVar";
    static monoFontFace = "RecVarMono";

    constructor() {
        super();
    }

    async connectedCallback() {
        this.style.cssText = `
            margin-top: 8px;
            margin-bottom: 4px;
            border-radius: 4px;
            max-width: 100%;
        `;

        this.ctx = this.getContext("2d");

        requestAnimationFrame(this.#drawLoop.bind(this));
    }

    #drawLoop() {
        this.ctx.font = "14px RecVar";
        this.draw();
        requestAnimationFrame(this.#drawLoop.bind(this));
    }

    // Override this!
    draw() {
        throw new ReferenceError("draw() must be overridden");
    }

    getTextPositionInBox(text, x, y, width, height, hAlign, vAlign) {
        let measurements = this.ctx.measureText(text);

        let leftX;
        switch (hAlign) {
            case "left":
                leftX = x;
                break;
            case "center":
                leftX = x + width / 2 - measurements.width / 2;
                break;
            case "right":
                leftX = x + width - measurements.width;
                break;
        }

        let textHeight = measurements.fontBoundingBoxAscent;
        let baselineY;
        switch (vAlign) {
            case "top":
                baselineY = y + textHeight;
                break;
            case "center":
                baselineY = y + height / 2 + textHeight / 2;
                break;
            case "bottom":
                baselineY = y + height;
                break;
        }

        return { leftX, baselineY };
    }

    get scaleInViewportX() {
        return this.clientWidth / this.width;
    }

    get scaleInViewportY() {
        return this.clientHeight / this.height;
    }

    getMousePositionFromEvent(event) {
        return {
            x: event.offsetX / this.scaleInViewportX,
            y: event.offsetY / this.scaleInViewportY,
        };
    }
}

export function defineFrame(elementName, claß) { // because `class` is a keyword.
    customElements.define(elementName, claß, { extends: "canvas" });
}

defineFrame("tairu--frame", Frame);
