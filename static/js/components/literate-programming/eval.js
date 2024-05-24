let outputIndex = 0;

export const jsConsole = console;

function stringifyValue(x) {
    jsConsole.log(typeof x, x instanceof Set);
    if (x === undefined) return "undefined";
    else if (x === null) return "null";
    else if (x instanceof Set) return `{${Array.from(x).map(stringifyValue).join(", ")}}`;
    else if (typeof x == "object") return x.toString();
    else return x + "";
}

// Overwrite globalThis.console with domConsole to redirect output to the DOM console.
// To always output to the JavaScript console regardless, use jsConsole.
export const domConsole = {
    log(...message) {
        postMessage({
            kind: "output",
            output: {
                kind: "console.log",
                message: [...message].map((x) => {
                    jsConsole.log(stringifyValue(x));
                    return stringifyValue(x);
                }),
            },
            outputIndex,
        });
    },
};

async function withTemporaryGlobalScope(callback) {
    let state = {
        oldValues: {},
        set(key, value) {
            this.oldValues[key] = globalThis[key];
            globalThis[key] = value;
        },
    };
    await callback(state);
    jsConsole.trace(state.oldValues, "bringing back old state");
    for (let key in state.oldValues) {
        globalThis[key] = state.oldValues[key];
    }
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

    try {
        for (let command of commands) {
            if (command.kind == "module") {
                let blobUrl = URL.createObjectURL(
                    new Blob([command.source], { type: "text/javascript" })
                );
                let module = await import(blobUrl);
                for (let exportedKey in module) {
                    globalThis[exportedKey] = module[exportedKey];
                }
            } else if (command.kind == "output") {
                if (newOutput != null) {
                    newOutput(outputIndex);
                }
            } else if (command.kind == "setOutputIndex") {
                outputIndex = command.outputIndex;
            }
        }
        postMessage({
            kind: "evalComplete",
        });
    } catch (err) {
        postMessage({
            kind: "output",
            output: {
                kind: "error",
                message: [err.toString()],
            },
            outputIndex,
        });
        if (error != null) {
            error();
        }
    }
    signalEvaluationComplete();
}
