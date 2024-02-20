// Emoji zoom-in functionality.

class EmojiTooltip extends HTMLElement {
    constructor(emoji, { onClosed }) {
        super();

        this.emoji = emoji;
        this.onClosed = onClosed;
    }

    connectedCallback() {
        this.role = "tooltip";

        this.image = new Image();
        this.image.src = this.emoji.src;

        this.description = document.createElement("p");
        this.description.textContent = `${this.emoji.emojiName}`;

        let emojiBoundingBox = this.emoji.getBoundingClientRect();
        this.style.left = `${emojiBoundingBox.left + emojiBoundingBox.width / 2}px`;
        this.style.top = `calc(${emojiBoundingBox.top}px + 1.5em)`;

        this.fullyOpaque = false;
        this.addEventListener("transitionend", event => {
            if (event.propertyName == "opacity") {
                this.fullyOpaque = !this.fullyOpaque;
                if (!this.fullyOpaque) {
                    this.onClosed();
                }
            }
        });
        // Timeout is zero because we just want to execute this later, to be definitely sure
        // the transition plays out.
        setTimeout(() => this.classList.add("transitioned-in"), 0);

        this.appendChild(this.image);
        this.appendChild(this.description);
    }

    close() {
        this.classList.remove("transitioned-in");
    }
}

customElements.define("th-emoji-tooltip", EmojiTooltip);

let emojiTooltips = null;
addEventListener("wheel", event => emojiTooltips.closeTooltips(event));

class EmojiTooltips extends HTMLElement {
    constructor() {
        super();
        this.tooltips = new Set();
        this.abortController = new AbortController();

    }

    connectedCallback() {
        emojiTooltips = this;
    }

    disconnectedCallback() {
        this.abortController.abort();
    }

    openTooltip(emoji) {
        let tooltip = new EmojiTooltip(emoji, {
            onClosed: () => {
                this.removeChild(tooltip);
                this.tooltips.delete(tooltip);
            },
        });

        this.appendChild(tooltip);
        this.tooltips.add(tooltip);

        return tooltip;
    }

    closeTooltip(tooltip) {
        tooltip.close();
    }

    closeTooltips() {
        for (let tooltip of this.tooltips) {
            console.log("close", this);

            tooltip.close();
        }
    }
}

customElements.define("th-emoji-tooltips", EmojiTooltips);

class Emoji extends HTMLImageElement {
    connectedCallback() {
        this.emojiName = this.title;

        // title makes the browser add a tooltip. We replace browser tooltips with our own,
        // so remove the title.
        this.title = "";

        this.addEventListener("mouseenter", () => this.openTooltip());
        this.addEventListener("mouseleave", () => this.closeTooltip());
        this.addEventListener("scroll", () => this.closeTooltip());
    }

    openTooltip() {
        this.tooltip = emojiTooltips.openTooltip(this);
    }

    closeTooltip() {
        emojiTooltips.closeTooltip(this.tooltip);
        this.tooltip = null;
    }
}

customElements.define("th-emoji", Emoji, { extends: "img" });
