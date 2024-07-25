import { CodeJar } from "treehouse/vendor/codejar.js";
import { compileSyntax, highlight } from "treehouse/components/literate-programming/highlight.js";

const loggingEnabled = false;
function log(...message) {
    if (loggingEnabled) {
        console.log("[lp]", ...message);
    }
}

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
            commands.push({
                kind: "module",
                source: frame.textContent,
                language: frame.language,
                kernelParameters: frame.kernelAttributes,
            });
        } else if (frame.mode == "output") {
            commands.push({ kind: "output" });
        }
    }

    return commands;
}

let compiledSyntaxes = new Map();

async function getCompiledSyntax(language) {
    if (compiledSyntaxes.has(language)) {
        return compiledSyntaxes.get(language);
    } else {
        let json = await (await fetch(TREEHOUSE_SYNTAX_URLS[language])).text();
        let compiled = compileSyntax(JSON.parse(json));
        compiledSyntaxes.set(language, compiled);
        return compiled;
    }
}

class InputMode {
    constructor(frame) {
        this.frame = frame;

        getCompiledSyntax(this.frame.language).then((syntax) => {
            this.syntax = syntax;
            this.highlight();
        });

        this.codeJar = CodeJar(frame, (frame) => this.highlight(frame));
        this.codeJar.onUpdate(() => {
            log(`${this.frame.programName} input frame ${this.frame.frameIndex} update`);
            for (let handler of frame.program.onChanged) {
                handler(frame.programName, frame.frameIndex);
            }
        });

        frame.addEventListener("click", (event) => event.preventDefault());
    }

    async highlight() {
        if (this.syntax == null) return;

        highlight(this.frame, this.syntax, (token, span) => {
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
        .map((x) => {
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

        this.currentEvaluation = new Promise((resolve) => resolve());
        this.completeCurrentEvaluation = () => {};
        this.evaluationEnqueued = false;

        this.iframe.contentWindow.treehouseSandboxInternals = { outputIndex: this.outputIndex };

        this.iframe.contentWindow.addEventListener("message", (event) => {
            log(
                `${this.frame.programName} output frame ${this.frame.frameIndex} (output index ${this.outputIndex}) recv`,
                event.data,
            );

            let message = event.data;
            if (message.kind == "ready") {
                this.evaluate();
            } else if (message.kind == "resize" && message.outputIndex == this.outputIndex) {
                this.resize();
            } else if (message.kind == "output" && message.outputIndex == this.outputIndex) {
                if (message.output.kind == "error") {
                    this.completeCurrentEvaluation();
                    this.error.textContent = messageOutputArrayToString(message.output.message);
                    this.iframe.classList.add("hidden");
                } else {
                    this.addOutput(message.output);
                }
            } else if (message.kind == "evalComplete") {
                this.completeCurrentEvaluation();
                this.error.textContent = "";
                this.flushConsoleClear();
            }
        });

        if (this.frame.placeholderImage != null) {
            this.frame.placeholderImage.classList.add("js", "loading");
        }

        this.frame.program.onChanged.push((_, inputFrameIndex) => {
            if (inputFrameIndex < this.frame.frameIndex) {
                this.evaluateWhenPossible();
            }
        });
    }

    evaluateWhenPossible() {
        if (!this.evaluationEnqueued) {
            this.evaluationEnqueued = true;
            this.currentEvaluation.then(() => {
                this.evaluate();
                this.evaluationEnqueued = false;
            });
        }
    }

    evaluate() {
        log(`${this.frame.programName} output frame ${this.frame.frameIndex} evaluate`);

        this.currentEvaluation = new Promise((resolve) => {
            this.completeCurrentEvaluation = resolve;
        });

        this.requestConsoleClear();
        this.iframe.contentWindow.postMessage({
            action: "eval",
            input: getLiterateProgramWorkerCommands(
                this.frame.programName,
                this.frame.frameIndex + 1,
            ),
        });
    }

    clearConsole() {
        if (this.frame.placeholderConsole != null) {
            this.frame.removeChild(this.frame.placeholderConsole);
            this.frame.placeholderConsole = null;
        }
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
            .map((x) => {
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
            this.frame.placeholderImage = null;
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
        this.language = this.getAttribute("data-language");
        this.programName = this.getAttribute("data-program");
        this.frameIndex = this.program.frames.length;
        this.program.frames.push(this);

        this.placeholderImage = this.getElementsByClassName("placeholder-image")[0];
        this.placeholderConsole = this.getElementsByClassName("placeholder-console")[0];

        this.kernelAttributes = {};
        for (let name of this.getAttributeNames()) {
            if (name.startsWith("k-")) {
                this.kernelAttributes[name] = this.getAttribute(name);
            }
        }

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
