let outputIndex = 0;

export function getOutputIndex() {
    return outputIndex;
}

async function withTemporaryGlobalScope(callback) {
    let state = {
        oldValues: {},
        set(key, value) {
            this.oldValues[key] = globalThis[key];
            globalThis[key] = value;
        }
    };
    await callback(state);
    for (let key in state.oldValues) {
        globalThis[key] = state.oldValues[key];
    }
}

let evaluationComplete = null;

export async function evaluate(commands, { start, success, error }) {
    if (evaluationComplete != null) {
        await evaluationComplete;
    }

    if (start != null) {
        start();
    }

    let signalEvaluationComplete;
    evaluationComplete = new Promise((resolve, _reject) => {
        signalEvaluationComplete = resolve;
    })

    outputIndex = 0;
    try {
        await withTemporaryGlobalScope(async scope => {
            for (let command of commands) {
                if (command.kind == "module") {
                    let blobUrl = URL.createObjectURL(new Blob([command.source], { type: "text/javascript" }));
                    let module = await import(blobUrl);
                    for (let exportedKey in module) {
                        scope.set(exportedKey, module[exportedKey]);
                    }
                } else if (command.kind == "output") {
                    ++outputIndex;
                }
            }
        });
        if (success != null) {
            success();
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

