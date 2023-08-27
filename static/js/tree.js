import { navigationMap } from "/navmap.js";

const branchStateKey = "treehouse.openBranches";
let branchState = JSON.parse(localStorage.getItem(branchStateKey)) || {};

function saveBranchIsOpen(branchID, state) {
    branchState[branchID] = state;
    localStorage.setItem(branchStateKey, JSON.stringify(branchState));
}

function branchIsOpen(branchID) {
    return branchState[branchID];
}

class Branch extends HTMLLIElement {
    constructor() {
        super();

        this.details = this.childNodes[0];
        this.innerUL = this.details.childNodes[1];

        let doPersist = !this.hasAttribute("data-th-do-not-persist");
        let isOpen = branchIsOpen(this.id);
        if (doPersist && isOpen !== undefined) {
            this.details.open = isOpen;
        }
        this.details.addEventListener("toggle", _ => {
            saveBranchIsOpen(this.id, this.details.open);
        });
    }
}

customElements.define("th-b", Branch, { extends: "li" });

class LinkedBranch extends Branch {
    static byLink = new Map();

    constructor() {
        super();

        this.linkedTree = this.getAttribute("data-th-link");
        LinkedBranch.byLink.set(this.linkedTree, this);

        this.loadingState = "notloaded";

        this.loadingText = document.createElement("p");
        {
            this.loadingText.className = "link-loading";
            let linkedTreeName = document.createElement("code");
            linkedTreeName.innerText = this.linkedTree;
            this.loadingText.append("Loading ", linkedTreeName, "...");
        }
        this.innerUL.appendChild(this.loadingText);

        // This produces a warning during static generation but we still want to handle that
        // correctly, as Branch saves the state in localStorage. Having an expanded-by-default
        // linked block can be useful in development.
        if (this.details.open) {
            this.loadTree("constructor");
        }

        this.details.addEventListener("toggle", _ => {
            if (this.details.open) {
                this.loadTree("toggle");
            }
        });
    }

    async loadTreePromise(_initiator) {
        try {
            let response = await fetch(`/${this.linkedTree}.html`);
            if (response.status == 404) {
                throw `Hmm, seems like the tree "${this.linkedTree}" does not exist.`;
            }

            let text = await response.text();
            let parser = new DOMParser();
            let linkedDocument = parser.parseFromString(text, "text/html");
            let main = linkedDocument.getElementsByTagName("main")[0];
            let ul = main.getElementsByTagName("ul")[0];

            this.loadingText.remove();
            this.innerUL.innerHTML = ul.innerHTML;
        } catch (error) {
            this.loadingText.innerText = error.toString();
        }
    }

    loadTree() {
        if (!this.loading) {
            this.loading = this.loadTreePromise();
        }
        return this.loading;
    }
}


customElements.define("th-b-linked", LinkedBranch, { extends: "li" });

function expandDetailsRecursively(element) {
    while (element && element.tagName != "MAIN") {
        if (element.tagName == "DETAILS") {
            element.open = true;
        }
        element = element.parentElement;
    }
}

function navigateToPage(page) {
    window.location.pathname = `/${page}.html`
}

async function navigateToBranch(fragment) {
    if (fragment.length == 0) {
        return;
    }

    let element = document.getElementById(fragment);
    if (element !== null) {
        // If the element is already loaded on the page, we're good.
        expandDetailsRecursively(element);
        window.location.hash = window.location.hash;
    } else {
        // The element is not loaded, we need to load the tree that has it.
        let parts = fragment.split(':');
        if (parts.length >= 2) {
            let [page, _id] = parts;
            let fullPath = navigationMap[page];
            if (Array.isArray(fullPath)) {
                // TODO: This logic will probably need to be upgraded at some point to support
                // navigation maps with roots other than index. Currently though only index is
                // generated so that doesn't matter.
                let [_root, ...path] = fullPath;
                if (path !== undefined) {
                    let isNotAtIndexHtml = window.location.pathname != "/index.html";
                    for (let linked of path) {
                        let branch = LinkedBranch.byLink.get(linked);

                        if (isNotAtIndexHtml && branch === undefined) {
                            navigateToPage("index");
                            return;
                        }

                        await branch.loadTree("navigateToBranch");
                        branch.details.open = true;
                    }
                    window.location.hash = window.location.hash;
                }
            } else {
                // In case the navigation map does not contain the given page, we can try
                // redirecting the user to a concrete page on the site.
                navigateToPage(page);
            }
        }
    }
}

async function navigateToCurrentBranch() {
    let location = window.location.hash.substring(1);
    navigateToBranch(location);
}

// When you click on a link, and the destination is within a <details> that is not expanded,
// expand the <details> recursively.
window.addEventListener("popstate", navigateToCurrentBranch);
addEventListener("DOMContentLoaded", navigateToCurrentBranch);
