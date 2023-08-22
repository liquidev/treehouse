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
    constructor() {
        super();

        this.linkedTree = this.getAttribute("data-th-link");

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
            this.loadTree();
        }

        this.details.addEventListener("toggle", _ => {
            if (this.details.open) {
                this.loadTree();
            }
        });
    }

    loadTree() {
        if (this.loadingState == "notloaded") {
            this.loadingState = "loading";

            fetch(`/${this.linkedTree}.html`)
                .then(response => {
                    if (response.status == 404) {
                        throw `Hmm, seems like the tree "${this.linkedTree}" does not exist.`;
                    }
                    return response.text();
                })
                .then(text => {
                    let parser = new DOMParser();
                    let linkedDocument = parser.parseFromString(text, "text/html");
                    let main = linkedDocument.getElementsByTagName("main")[0];
                    let ul = main.getElementsByTagName("ul")[0];

                    this.loadingText.remove();
                    this.innerUL.innerHTML = ul.innerHTML;

                    this.loadingState = "loaded";
                })
                .catch(error => {
                    this.loadingText.innerText = error.toString();
                    this.loadingState = "error";
                });
        }
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

// When you click on a link, and the destination is within a <details> that is not expanded,
// expand the <details> recursively.
window.addEventListener("popstate", _ => {
    let element = document.getElementById(window.location.hash.substring(1));
    if (element !== undefined) {
        // If the element is already loaded on the page, we're good.
        expandDetailsRecursively(element);
        window.location.hash = window.location.hash;
    }
})
