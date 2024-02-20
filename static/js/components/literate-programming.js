import { CodeJar } from "../vendor/codejar.js";
import { compileSyntax, highlight } from "./literate-programming/highlight.js";

let literatePrograms = new Map();

function getLiterateProgram(name) {
    if (literatePrograms.get(name) == null) {
        literatePrograms.set(name, {
            frames: [],
            onChanged: [],

            outputCount: 0,

            nextOutputIndex() {
                return this.outputCount++;
            },
        });
    }
    return literatePrograms.get(name);
}

function getLiterateProgramWorkerCommands(name, count) {
    let commands = [];
    let literateProgram = getLiterateProgram(name);

    for (let i = 0; i < count; ++i) {
        let frame = literateProgram.frames[i];
        if (frame.mode == "input") {
            commands.push({ kind: "module", source: frame.textContent });
        } else if (frame.mode == "output") {
            commands.push({ kind: "output" });
        }
    }

    return commands;
}

class InputMode {
    static JAVASCRIPT = compileSyntax({
        patterns: [
            { regex: /\/\/.*/, as: "comment" },
            { regex: /\/\*.*?\*\//ms, as: "comment" },
            { regex: /[A-Z_][a-zA-Z0-9_]*/, as: "keyword2" },
            { regex: /[a-zA-Z_][a-zA-Z0-9_]*(?=\()/, as: "function" },
            { regex: /[a-zA-Z_][a-zA-Z0-9_]*/, as: "identifier" },
            { regex: /0[bB][01_]+n?/, as: "literal" },
            { regex: /0[oO][0-7_]+n?/, as: "literal" },
            { regex: /0[xX][0-9a-fA-F_]+n?/, as: "literal" },
            { regex: /[0-9_]+n/, as: "literal" },
            { regex: /[0-9_]+(\.[0-9_]*([eE][-+]?[0-9_]+)?)?/, as: "literal" },
            { regex: /'(\\'|[^'])*'/, as: "string" },
            { regex: /"(\\"|[^"])*"/, as: "string" },
            { regex: /`(\\`|[^"])*`/, as: "string" },
            // TODO: RegExp literals?
            { regex: /[+=/*^%<>!~|&\.?:-]+/, as: "operator" },
            { regex: /[,;]/, as: "punct" },
        ],
        keywords: new Map([
            ["as", { into: "keyword1", onlyReplaces: "identifier" }],
            ["async", { into: "keyword1", onlyReplaces: "identifier" }],
            ["await", { into: "keyword1" }],
            ["break", { into: "keyword1" }],
            ["case", { into: "keyword1" }],
            ["catch", { into: "keyword1" }],
            ["class", { into: "keyword1" }],
            ["const", { into: "keyword1" }],
            ["continue", { into: "keyword1" }],
            ["debugger", { into: "keyword1" }],
            ["default", { into: "keyword1" }],
            ["delete", { into: "keyword1" }],
            ["do", { into: "keyword1" }],
            ["else", { into: "keyword1" }],
            ["export", { into: "keyword1" }],
            ["extends", { into: "keyword1" }],
            ["finally", { into: "keyword1" }],
            ["for", { into: "keyword1" }],
            ["from", { into: "keyword1", onlyReplaces: "identifier" }],
            ["function", { into: "keyword1" }],
            ["get", { into: "keyword1", onlyReplaces: "identifier" }],
            ["if", { into: "keyword1" }],
            ["import", { into: "keyword1" }],
            ["in", { into: "keyword1" }],
            ["instanceof", { into: "keyword1" }],
            ["let", { into: "keyword1" }],
            ["new", { into: "keyword1" }],
            ["of", { into: "keyword1", onlyReplaces: "identifier" }],
            ["return", { into: "keyword1" }],
            ["set", { into: "keyword1", onlyReplaces: "identifier" }],
            ["static", { into: "keyword1" }],
            ["switch", { into: "keyword1" }],
            ["throw", { into: "keyword1" }],
            ["try", { into: "keyword1" }],
            ["typeof", { into: "keyword1" }],
            ["var", { into: "keyword1" }],
            ["void", { into: "keyword1" }],
            ["while", { into: "keyword1" }],
            ["with", { into: "keyword1" }],
            ["yield", { into: "keyword1" }],

            ["super", { into: "keyword2" }],
            ["this", { into: "keyword2" }],

            ["false", { into: "literal" }],
            ["true", { into: "literal" }],
            ["undefined", { into: "literal" }],
            ["null", { into: "literal" }],
        ]),
    })

    constructor(frame) {
        this.frame = frame;

        InputMode.highlight(frame);
        this.codeJar = CodeJar(frame, InputMode.highlight);
        this.codeJar.onUpdate(() => {
            for (let handler of frame.program.onChanged) {
                handler(frame.programName);
            }
        })

        frame.addEventListener("click", event => event.preventDefault());
    }

    static highlight(frame) {
        highlight(frame, InputMode.JAVASCRIPT, (token, span) => {
            if (token.kind == "keyword1" && token.string == "export") {
                // This is something a bit non-obvious about the treehouse's literate programs
                // so let's document it.
                span.classList.add("export");
                span.title = "This item is exported and visible in code blocks that follow";
            }
        });
    }
}

function messageOutputArrayToString(output) {
    return output
        .map(x => {
            if (typeof x === "object") return JSON.stringify(x);
            else return x + "";
        })
        .join(" ");
}

class OutputMode {
    constructor(frame) {
        this.frame = frame;

        this.outputIndex = this.frame.program.nextOutputIndex();

        this.console = document.createElement("pre");
        this.console.classList.add("console");
        this.frame.appendChild(this.console);
        this.clearConsoleOnNextOutput = false;

        this.error = document.createElement("pre");
        this.error.classList.add("error");
        this.frame.appendChild(this.error);

        this.iframe = document.createElement("iframe");
        this.iframe.classList.add("hidden");
        this.iframe.src = `${TREEHOUSE_SITE}/sandbox`;
        this.frame.appendChild(this.iframe);

        this.iframe.contentWindow.treehouseSandboxInternals = { outputIndex: this.outputIndex };

        this.iframe.contentWindow.addEventListener("message", event => {
            let message = event.data;
            if (message.kind == "ready") {
                this.evaluate();
            } else if (message.kind == "resize" && message.outputIndex == this.outputIndex) {
                this.resize();
            } else if (message.kind == "output" && message.outputIndex == this.outputIndex) {
                if (message.output.kind == "error") {
                    this.error.textContent = messageOutputArrayToString(message.output.message);
                    this.iframe.classList.add("hidden");
                } else {
                    this.addOutput(message.output);
                }
            } else if (message.kind == "evalComplete") {
                this.error.textContent = "";
                this.flushConsoleClear();
            }
        });

        this.frame.placeholderImage.classList.add("loading");

        this.frame.program.onChanged.push(_ => this.evaluate());
    }

    evaluate() {
        this.requestConsoleClear();
        this.iframe.contentWindow.postMessage({
            action: "eval",
            input: getLiterateProgramWorkerCommands(this.frame.programName, this.frame.frameIndex + 1),
        });
    }

    clearConsole() {
        this.console.replaceChildren();
    }

    requestConsoleClear() {
        this.clearConsoleOnNextOutput = true;
    }

    flushConsoleClear() {
        if (this.clearConsoleOnNextOutput) {
            this.clearConsole();
            this.clearConsoleOnNextOutput = false;
        }
    }

    addOutput(output) {
        this.flushConsoleClear();

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

        this.console.appendChild(line);
    }

    resize() {
        // iframe cannot be `display: none` to get its scrollWidth/scrollHeight.
        this.iframe.classList.remove("hidden");

        if (this.frame.placeholderImage != null) {
            // Fade the iframe in after it becomes visible, and remove the image.
            setTimeout(() => this.iframe.classList.add("loaded"), 0);
            this.frame.removeChild(this.frame.placeholderImage);
        } else {
            // If there is no image, don't do the fade in.
            this.iframe.classList.add("loaded");
        }

        let width = this.iframe.contentDocument.body.scrollWidth;
        let height = this.iframe.contentDocument.body.scrollHeight;

        if (width == 0 || height == 0) {
            this.iframe.classList.add("hidden");
        } else {
            this.iframe.width = width;
            this.iframe.height = height;
        }
    }
}

class LiterateProgram extends HTMLElement {
    connectedCallback() {
        this.programName = this.getAttribute("data-program");
        this.frameIndex = this.program.frames.length;
        this.program.frames.push(this);

        this.placeholderImage = this.getElementsByClassName("placeholder")[0];

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
