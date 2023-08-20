class LinkedBranch extends HTMLLIElement {
    constructor() {
        super();

        this.linkedTree = this.getAttribute("data-th-link");

        this.details = this.childNodes[0];
        this.innerUL = this.details.childNodes[1];

        this.state = "notloaded";

        this.loadingText = document.createElement("p");
        {
            this.loadingText.className = "link-loading";
            let linkedTreeName = document.createElement("code");
            linkedTreeName.innerText = this.linkedTree;
            this.loadingText.append("Loading ", linkedTreeName, "...");
        }
        this.innerUL.appendChild(this.loadingText);

        // This produces a warning during static generation but we still want to handle that
        // correctly. Having an expanded-by-default linked block can be useful in development.
        if (this.details.open) {
            this.loadTree();
        }

        this.details.addEventListener("toggle", event => {
            if (this.details.open) {
                this.loadTree();
            }
        });
    }

    loadTree() {
        if (this.state == "notloaded") {
            this.state = "loading";

            fetch(`/${this.linkedTree}.html`)
                .then(request => request.text())
                .then(text => {
                    let parser = new DOMParser();
                    let linkedDocument = parser.parseFromString(text, "text/html");
                    let main = linkedDocument.getElementsByTagName("main")[0];
                    let ul /*: Element */ = main.getElementsByTagName("ul")[0];
                    console.log(ul);

                    this.loadingText.remove();

                    for (let i = 0; i < ul.childNodes.length; ++i) {
                        this.innerUL.appendChild(ul.childNodes[i]);
                    }
                })
                .catch(error => {
                    this.loadingText.innerText = error.toString();
                });
        }
    }
}

customElements.define("th-linked-branch", LinkedBranch, { extends: "li" });
