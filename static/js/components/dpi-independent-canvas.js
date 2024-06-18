export class DpiIndependentCanvas {
    /**
     * @param {Object} params
     * @param {(dpic: DpiIndependentCanvas, canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) => void} params.draw */
    constructor({ width, height, context, draw }) {
        this.width = width;
        this.height = height;

        this.canvas = document.createElement("canvas");
        this.ctx = this.canvas.getContext(context);

        this.drawFunction = draw;

        // Trigger redraw when pixel ratio is updated.
        let remove = null;
        let updatePixelRatio = () => {
            if (remove != null) {
                remove();
            }

            let mediaQueryString = `(resolution: ${window.devicePixelRatio}dppx)`;
            let media = window.matchMedia(mediaQueryString);
            media.addEventListener("change", updatePixelRatio);
            remove = () =>
                media.removeEventListener("change", updatePixelRatio);

            this.#updateSize();
            requestAnimationFrame(() => this.#redraw());
        };
        updatePixelRatio();
    }

    #redraw() {
        this.ctx.save();
        this.ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
        this.drawFunction(this, this.canvas, this.ctx);
        this.ctx.restore();
    }

    #updateSize() {
        this.canvas.width = this.width * window.devicePixelRatio;
        this.canvas.height = this.height * window.devicePixelRatio;
        this.canvas.style.width = `${this.width}px`;
        this.canvas.style.height = `${this.height}px`;
    }

    #redrawLoop() {
        requestAnimationFrame(() => {
            this.#redraw();
            if (this.animating) {
                this.#redrawLoop();
            }
        });
    }

    startAnimating() {
        this.animating = true;
        this.#redrawLoop();
    }

    stopAnimating() {
        this.animating = false;
    }
}
