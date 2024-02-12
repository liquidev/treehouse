// A frameworking class assigning some CSS classes to the canvas to make it integrate nicer with CSS.
class Frame extends HTMLCanvasElement {
    constructor() {
        super();

        this.style.cssText = `

        `;
    }

    // Override this!
    draw() { }
}
