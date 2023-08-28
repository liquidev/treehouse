// Emoji zoom-in functionality.

class Emoji extends HTMLImageElement {
    constructor() {
        super();

        this.wrapper = document.createElement("span");
        this.wrapper.className = "emoji-wrapper";
        this.replaceWith(this.wrapper);
        this.wrapper.appendChild(this);

        this.enlarged = new Image();
        this.enlarged.src = this.src;

        this.titleElement = document.createElement("p");
        this.titleElement.innerText = this.title;

        this.tooltip = document.createElement("div");
        this.tooltip.className = "emoji-tooltip";
        this.tooltip.appendChild(this.enlarged);
        this.tooltip.appendChild(this.titleElement);

        this.wrapper.appendChild(this.tooltip);

        this.alt = this.title;
        this.title = "";
    }
}

customElements.define("th-emoji", Emoji, { extends: "img" });
