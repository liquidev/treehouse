import { CodeJar } from "../vendor/codejar.js";

let literatePrograms = new Map();

function getLiterateProgram(name) {
    if (literatePrograms.get(name) == null) {
        literatePrograms.set(name, {
            frames: [],
            onChanged: [],

            outputCount: 0,

            nextOutputIndex() {
                let index = this.outputCount;
                ++this.outputCount;
                return index;
            }
        });
    }
    return literatePrograms.get(name);
}

function getLiterateProgramWorkerCommands(name) {
    let commands = [];
    let literateProgram = getLiterateProgram(name);
    for (let frame of literateProgram.frames) {
        if (frame.mode == "input") {
            commands.push({ kind: "module", source: frame.textContent });
        } else if (frame.mode == "output") {
            commands.push({ kind: "output", expected: frame.textContent });
        }
    }
    return commands;
}

class InputMode {
    constructor(frame) {
        this.frame = frame;

        this.codeJar = CodeJar(frame, InputMode.highlight);
        this.codeJar.onUpdate(() => {
            for (let handler of frame.program.onChanged) {
                handler(frame.programName);
            }
        })

        frame.addEventListener("click", event => event.preventDefault());
    }

    static highlight(frame) {
        // TODO: Syntax highlighting
    }
}

class OutputMode {
    constructor(frame) {
        this.clearResultsOnNextOutput = false;

        this.frame = frame;

        this.frame.program.onChanged.push(_ => this.evaluate());
        this.outputIndex = this.frame.program.nextOutputIndex();

        this.evaluate();
    }

    evaluate() {
        // This is a small bit of debouncing. If we cleared the output right away, the page would
        // jitter around irritatingly.
        this.clearResultsOnNextOutput = true;

        if (this.worker != null) {
            this.worker.terminate();
        }
        this.worker = new Worker(`${TREEHOUSE_SITE}/static/js/components/literate-programming/worker.js`, {
            type: "module",
            name: `evaluate LiterateOutput ${this.frame.programName}`
        });

        this.worker.addEventListener("message", event => {
            let message = event.data;
            if (message.kind == "evalComplete") {
                this.worker.terminate();
            } else if (message.kind == "output" && message.outputIndex == this.outputIndex) {
                this.addOutput(message.output);
            }
        });

        this.worker.postMessage({
            action: "eval",
            input: getLiterateProgramWorkerCommands(this.frame.programName),
        });
    }

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

        // One day this will be more fancy. Today is not that day.
        line.textContent = output.message
            .map(x => {
                if (typeof x === "object") return JSON.stringify(x);
                else return x + "";
            })
            .join(" ");

        if (output.kind == "result") {
            let returnValueText = document.createElement("span");
            returnValueText.classList.add("return-value");
            returnValueText.textContent = "Return value: ";
            line.insertBefore(returnValueText, line.firstChild);
        }

        this.frame.appendChild(line);
    }

    clearResults() {
        this.frame.replaceChildren();
    }
}

class LiterateProgram extends HTMLElement {
    connectedCallback() {
        this.programName = this.getAttribute("data-program");
        this.program.frames.push(this);

        this.mode = this.getAttribute("data-mode");
        if (this.mode == "input") {
            this.modeImpl = new InputMode(this);
        } else if (this.mode == "output") {
            this.modeImpl = new OutputMode(this);
        }
    }

    get program() {
        return getLiterateProgram(this.programName);
    }
}

customElements.define("th-literate-program", LiterateProgram);
