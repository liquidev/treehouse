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

export async function evaluate(commands) {
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
    } catch (error) {
        postMessage({
            kind: "output",
            output: {
                kind: "error",
                message: [error.toString()],
            },
            outputIndex,
        });
    }
}

