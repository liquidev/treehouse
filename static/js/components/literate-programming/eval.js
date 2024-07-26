let outputIndex = 0;

export const jsConsole = console;

const loggingEnabled = false;
function log(...message) {
    if (loggingEnabled) {
        jsConsole.log("[eval]", ...message);
    }
}

// Overwrite globalThis.console with domConsole to redirect output to the DOM console.
// To always output to the JavaScript console regardless, use jsConsole.
export const domConsole = {
    log(...message) {
        postMessage({
            kind: "output",
            output: {
                kind: "console.log",
                message: [...message],
            },
            outputIndex,
        });
    },
};

export async function defaultEvalModule(_state, source, language, _params) {
    if (language == "javascript") {
        let blobUrl = URL.createObjectURL(new Blob([source], { type: "text/javascript" }));
        let module = await import(blobUrl);
        for (let exportedKey in module) {
            globalThis[exportedKey] = module[exportedKey];
        }
        return _state;
    } else {
        return null;
    }
}

let kernel = {
    evalModule: defaultEvalModule,
};

export function getKernel() {
    return kernel;
}

let evaluationComplete = null;

export async function evaluate(commands, { error, newOutput }) {
    if (evaluationComplete != null) {
        await evaluationComplete;
    }

    let signalEvaluationComplete;
    evaluationComplete = new Promise((resolve, _reject) => {
        signalEvaluationComplete = resolve;
    });

    outputIndex = 0;
    try {
        let kernelState = {};
        for (let command of commands) {
            log(`frame ${treehouseSandboxInternals.outputIndex} module`, command);
            if (command.kind == "module") {
                await kernel.evalModule(
                    kernelState,
                    command.source,
                    command.language,
                    command.kernelParameters,
                );
            } else if (command.kind == "output") {
                if (newOutput != null) {
                    newOutput(outputIndex);
                }
                ++outputIndex;
            }
        }
        log(`frame ${treehouseSandboxInternals.outputIndex} evalComplete`);
        postMessage({
            kind: "evalComplete",
        });
    } catch (err) {
        log(`frame ${treehouseSandboxInternals.outputIndex} error`, err);
        postMessage({
            kind: "output",
            output: {
                kind: "error",
                message: [
                    err.stack.length > 0 ? err.toString() + "\n\n" + err.stack : err.toString(),
                ],
            },
            outputIndex,
        });
        if (error != null) {
            error();
        }
    }
    signalEvaluationComplete();
}
