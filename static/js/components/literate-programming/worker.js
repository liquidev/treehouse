let outputIndex = 0;

let debugLog = console.log;

globalThis.console = {
    log(...message) {
        postMessage({
            kind: "output",
            output: {
                kind: "log",
                message: [...message],
            },
            outputIndex,
        });
    }
};

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

addEventListener("message", async event => {
    let message = event.data;
    if (message.action == "eval") {
        outputIndex = 0;
        try {
            await withTemporaryGlobalScope(async scope => {
                for (let command of message.input) {
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

        postMessage({
            kind: "evalComplete",
        });
    }
});
