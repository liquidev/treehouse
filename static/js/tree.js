// This is definitely not a three.js ripoff.

import { navigationMap } from "/navmap.js";
import * as ulid from './ulid.js';

/* Branch persistence */

const branchStateKey = "treehouse.openBranches";
let branchState = JSON.parse(sessionStorage.getItem(branchStateKey)) || {};

function saveBranchIsOpen(branchID, state) {
    branchState[branchID] = state;
    sessionStorage.setItem(branchStateKey, JSON.stringify(branchState));
}

function branchIsOpen(branchID) {
    return branchState[branchID];
}

class Branch extends HTMLLIElement {
    static branchesByNamedID = new Map();

    constructor() {
        super();

        this.isLeaf = this.classList.contains("leaf");

        this.details = this.childNodes[0];
        this.innerUL = this.details.childNodes[1];

        if (this.isLeaf) {
            this.contentContainer = this.childNodes[0];
        } else {
            this.contentContainer = this.details.childNodes[0];
        }
        this.bulletPoint = this.contentContainer.childNodes[0];
        this.branchContent = this.contentContainer.childNodes[1];
        this.buttonBar = this.contentContainer.childNodes[2];

        let doPersist = !this.hasAttribute("data-th-do-not-persist");
        let isOpen = branchIsOpen(this.id);
        if (doPersist && isOpen !== undefined) {
            this.details.open = isOpen;
        }
        if (!this.isLeaf) {
            this.details.addEventListener("toggle", _ => {
                saveBranchIsOpen(this.id, this.details.open);
            });
        }

        let namedID = this.id.split(':')[1];
        Branch.branchesByNamedID.set(namedID, this);

        if (ulid.isCanonicalUlid(namedID)) {
            let timestamp = ulid.getTimestamp(namedID);
            let date = document.createElement("span");
            date.classList.add("branch-date");
            date.innerText = timestamp.toLocaleDateString();
            this.buttonBar.insertBefore(date, this.buttonBar.firstChild);
        }
    }
}

customElements.define("th-b", Branch, { extends: "li" });

/* Linked branches */

class LinkedBranch extends Branch {
    static byLink = new Map();

    constructor() {
        super();

        this.linkedTree = this.getAttribute("data-th-link");
        LinkedBranch.byLink.set(this.linkedTree, this);

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
            let response = await fetch(`${TREEHOUSE_SITE}/${this.linkedTree}.html`);
            if (response.status == 404) {
                throw `Hmm, seems like the tree "${this.linkedTree}" does not exist.`;
            }

            let text = await response.text();
            let parser = new DOMParser();
            let linkedDocument = parser.parseFromString(text, "text/html");
            let main = linkedDocument.getElementsByTagName("main")[0];
            let ul = main.getElementsByTagName("ul")[0];
            let styles = main.getElementsByTagName("link");
            let scripts = main.getElementsByTagName("script");


            this.loadingText.remove();
            this.innerUL.innerHTML = ul.innerHTML;

            this.append(...styles);
            for (let script of scripts) {
                // No need to await for the import because we don't use the resulting module.
                // Just fire and forger 💀
                // and let them run in parallel.
                import(script.src);
            }
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

/* Fragment navigation */

let rehashing = false;
function rehash() { // https://www.youtube.com/watch?v=Tv1SYqLllKI
    if (!rehashing) {
        rehashing = true;
        let hash = window.location.hash;
        if (hash.length > 0) {
            window.location.hash = "";
            window.location.hash = hash;
        }
        rehashing = false;
    }
}

function expandDetailsRecursively(element) {
    while (element && element.tagName != "MAIN") {
        if (element.tagName == "DETAILS") {
            element.open = true;
        }
        element = element.parentElement;
    }
}

function navigateToPage(page) {
    console.log(page);
    window.location.pathname = `${page}.html`
}

async function navigateToBranch(fragment) {
    if (fragment.length == 0) return;

    console.log(`nagivating to branch: ${fragment}`);

    let element = document.getElementById(fragment);
    if (element !== null) {
        // If the element is already loaded on the page, we're good.
        expandDetailsRecursively(element);
        rehash();
    } else {
        // The element is not loaded, we need to load the tree that has it.
        console.log("element is not loaded");
        let parts = fragment.split(':');
        if (parts.length >= 2) {
            let [page, _id] = parts;
            let fullPath = navigationMap[page];
            if (Array.isArray(fullPath)) {
                // TODO: This logic will probably need to be upgraded at some point to support
                // navigation maps with roots other than index. Currently though only index is
                // generated so that doesn't matter.
                let [_root, ...path] = fullPath;
                console.log(`_root: ${_root}, path: ${path}`)
                if (path !== undefined) {
                    let isNotAtIndexHtml =
                        window.location.pathname != "" &&
                        window.location.pathname != "/" &&
                        window.location.pathname != "/index.html";
                    let lastBranch = null;
                    for (let linked of path) {
                        let branch = LinkedBranch.byLink.get(linked);

                        if (isNotAtIndexHtml && branch === undefined) {
                            navigateToPage("index");
                            return;
                        }

                        await branch.loadTree("navigateToBranch");
                        lastBranch = branch;
                    }
                    if (lastBranch != null) {
                        expandDetailsRecursively(lastBranch.details);
                    }
                    rehash();
                }
            } else {
                // In case the navigation map does not contain the given page, we can try
                // redirecting the user to a concrete page on the site.
                navigateToPage(page);
            }
        }
    }
}

function getCurrentlyHighlightedBranch() {
    if (window.location.pathname == "/b" && window.location.search.length > 0) {
        let shortID = window.location.search.substring(1);
        return Branch.branchesByNamedID.get(shortID).id;
    } else {
        return window.location.hash.substring(1);
    }
}

async function navigateToCurrentBranch() {
    await navigateToBranch(getCurrentlyHighlightedBranch());
}

// When you click on a link, and the destination is within a <details> that is not expanded,
// expand the <details> recursively.
window.addEventListener("popstate", navigateToCurrentBranch);
addEventListener("DOMContentLoaded", navigateToCurrentBranch);

// When you enter the website through a link someone sent you, it would be nice if the linked branch
// got expanded by default.
async function expandLinkedBranch() {
    let currentlyHighlightedBranch = getCurrentlyHighlightedBranch();
    if (currentlyHighlightedBranch.length > 0) {
        let linkedBranch = document.getElementById(currentlyHighlightedBranch);
        if (linkedBranch.children.length > 0 && linkedBranch.children[0].tagName == "DETAILS") {
            expandDetailsRecursively(linkedBranch.children[0]);
        }
    }
}

addEventListener("DOMContentLoaded", expandLinkedBranch);

async function highlightCurrentBranch() {
    let branch = document.getElementById(getCurrentlyHighlightedBranch());
    if (branch != null) {
        branch.classList.add("target");
    }
}

addEventListener("DOMContentLoaded", highlightCurrentBranch);
