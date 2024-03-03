// Emoji zoom-in functionality.

import { addSpell } from "treehouse/spells.js";

class EmojiTooltip extends HTMLElement {
    constructor(emoji, element, { onClosed }) {
        super();

        this.emoji = emoji;
        this.emojiElement = element;
        this.onClosed = onClosed;
    }

    connectedCallback() {
        this.role = "tooltip";

        this.image = new Image();
        this.image.src = this.emojiElement.src;

        this.description = document.createElement("p");
        this.description.textContent = `${this.emoji.emojiName}`;

        let emojiBoundingBox = this.emojiElement.getBoundingClientRect();
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

class EmojiTooltips extends HTMLElement {
    constructor() {
        super();
        this.tooltips = new Set();
        this.abortController = new AbortController();
    }

    connectedCallback() {
        emojiTooltips = this;

        addEventListener(
            "wheel",
            event => emojiTooltips.closeTooltips(event),
            { signal: this.abortController.signal },
        );
    }

    disconnectedCallback() {
        this.abortController.abort();
    }

    openTooltip(emoji, element) {
        let tooltip = new EmojiTooltip(emoji, element, {
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
            tooltip.close();
        }
    }
}

customElements.define("th-emoji-tooltips", EmojiTooltips);

class Emoji {
    constructor(element) {
        this.emojiName = element.title;

        // title makes the browser add a tooltip. We replace browser tooltips with our own,
        // so remove the title.
        element.title = "";

        element.addEventListener("mouseenter", () => this.openTooltip(element));
        element.addEventListener("mouseleave", () => this.closeTooltip());
        element.addEventListener("scroll", () => this.closeTooltip());
    }

    openTooltip(element) {
        this.tooltip = emojiTooltips.openTooltip(this, element);
    }

    closeTooltip() {
        emojiTooltips.closeTooltip(this.tooltip);
        this.tooltip = null;
    }
}

addSpell("emoji", Emoji);
