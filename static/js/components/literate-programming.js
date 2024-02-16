import { CodeJar } from "../vendor/codejar.js";

let literatePrograms = new Map();

function getLiterateProgram(name) {
    if (literatePrograms.get(name) == null) {
        literatePrograms.set(name, {
            editors: [],
            onChanged: [],
        });
    }
    return literatePrograms.get(name);
}

function getLiterateProgramSourceCode(name) {
    let sources = [];
    let literateProgram = getLiterateProgram(name);
    for (let editor of literateProgram.editors) {
        sources.push(editor.textContent);
    }
    return sources.join("\n");
}

class LiterateEditor extends HTMLElement {
    constructor() {
        super();
    }

    connectedCallback() {
        this.literateProgramName = this.getAttribute("data-program");
        getLiterateProgram(this.literateProgramName).editors.push(this);

        this.codeJar = CodeJar(this, LiterateEditor.highlight);
        this.codeJar.onUpdate(() => {
            let literateProgram = getLiterateProgram(this.literateProgramName);
            for (let handler of literateProgram.onChanged) {
                handler(this.literateProgramName);
            }
        })

        this.addEventListener("click", event => event.preventDefault());
    }

    static highlight(editor) {
        // TODO: Syntax highlighting
    }
}

customElements.define("th-literate-editor", LiterateEditor);

function debounce(callback, timeout) {
    let timeoutId = 0;
    return (...args) => {
        clearTimeout(timeout);
        timeoutId = window.setTimeout(() => callback(...args), timeout);
    };
}

class LiterateOutput extends HTMLElement {
    constructor() {
        super();

        this.clearResultsOnNextOutput = false;
    }

    connectedCallback() {
        this.literateProgramName = this.getAttribute("data-program");
        this.evaluate();

        getLiterateProgram(this.literateProgramName).onChanged.push(_ => this.evaluate());
    }

    evaluate = () => {
        // This is a small bit of debouncing. If we cleared the output right away, the page would
        // jitter around irritatingly
        this.clearResultsOnNextOutput = true;

        if (this.worker != null) {
            this.worker.terminate();
        }
        this.worker = new Worker(`${TREEHOUSE_SITE}/static/js/components/literate-programming/worker.js`, {
            type: "module",
            name: `evaluate LiterateOutput ${this.literateProgramName}`
        });

        this.worker.addEventListener("message", event => {
            let message = event.data;
            if (message.kind == "evalComplete") {
                this.worker.terminate();
            } else if (message.kind == "output") {
                this.addOutput(message.output);
            }
        });

        this.worker.postMessage({
            action: "eval",
            input: getLiterateProgramSourceCode(this.literateProgramName),
        });
    };

    addOutput(output) {
        if (this.clearResultsOnNextOutput) {
            this.clearResultsOnNextOutput = false;
            this.clearResults();
        }

        // Don't show anything if the function didn't return a value.
        if (output.kind == "result" && output.message[0] === undefined) return;

        let line = document.createElement("code");

        line.classList.add("output");
        line.classList.add(output.kind);

        line.textContent = output.message.map(x => {
            if (typeof x === "object") return JSON.stringify(x);
            else return x + "";
        }).join(" ");

        if (output.kind == "result") {
            let returnValueText = document.createElement("span");
            returnValueText.classList.add("return-value");
            returnValueText.textContent = "Return value: ";
            line.insertBefore(returnValueText, line.firstChild);
        }

        this.appendChild(line);
    }

    clearResults() {
        this.replaceChildren();
    }
}

customElements.define("th-literate-output", LiterateOutput);
