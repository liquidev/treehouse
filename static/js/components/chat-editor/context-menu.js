export class ContextMenu extends HTMLElement {
    constructor(mouseEvent) {
        super();
        this.mouseEvent = mouseEvent;
    }

    connectedCallback() {
        this.classList.add("th-context-menu");

        window.addEventListener("mousedown", (event) => {
            let bounds = this.getBoundingClientRect();
            let isInBounds =
                event.clientX >= bounds.left &&
                event.clientX <= bounds.right &&
                event.clientY >= bounds.top &&
                event.clientY <= bounds.bottom;
            if (!isInBounds) {
                this.close();
            }
        });
    }

    close() {
        if (this.parentNode != null) {
            this.parentNode.removeChild(this);
        }
    }
}

export let contextMenus = null;

class ContextMenus extends HTMLElement {
    connectedCallback() {
        console.assert(
            contextMenus == null,
            "there must only be one th-context-menus in a document"
        );
        contextMenus = this;

        this.container = this.appendChild(document.createElement("div"));
    }

    openAt(contextMenu, x, y) {
        // NOTE: Need to append the context menu as a child before we get its bounding client rect.
        // Otherwise its layout isn't yet calculated and the bounding client rect will
        // be unavailable.
        this.container.appendChild(contextMenu);

        let screenBounds = this.getBoundingClientRect();
        let menuBounds = contextMenu.getBoundingClientRect();

        x -= screenBounds.left;
        y -= screenBounds.top;
        x = Math.min(x, screenBounds.right - menuBounds.width);
        y = Math.min(y, screenBounds.bottom - menuBounds.height);

        contextMenu.style.left = `${x}px`;
        contextMenu.style.top = `${y}px`;

        return contextMenu;
    }

    openAtCursor(contextMenu) {
        return this.openAt(
            contextMenu,
            contextMenu.mouseEvent.clientX,
            contextMenu.mouseEvent.clientY
        );
    }

    openAtDropdown(contextMenu, dropdown) {
        let bounds = dropdown.getBoundingClientRect();
        return this.openAt(contextMenu, bounds.left, bounds.bottom);
    }
}

customElements.define("th-context-menus", ContextMenus);
