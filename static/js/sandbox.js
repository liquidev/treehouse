export const internals = {
    body: document.createElement("body"),

    resetBody() {
        this.body.replaceChildren();
    }
};

export function body() {
    return internals.body;
}

export function addElement(element) {
    body().appendChild(element);
}

export class Sketch {
    constructor(width, height) {
        this.canvas = document.createElement("canvas");
        this.canvas.width = width;
        this.canvas.height = height;
        this.ctx = this.canvas.getContext("2d");

        addElement(this.canvas);
    }

    animate(draw) {
        let animationCallback;
        animationCallback = () => {
            draw();
            requestAnimationFrame(animationCallback);
        };
        animationCallback();
    }
}
