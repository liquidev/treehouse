import { evaluate, getOutputIndex } from "./eval.js";

let debugLog = console.log;

globalThis.console = {
    log(...message) {
        postMessage({
            kind: "output",
            output: {
                kind: "log",
                message: [...message],
            },
            outputIndex: getOutputIndex(),
        });
    }
};

addEventListener("message", async event => {
    let message = event.data;
    if (message.action == "eval") {
        evaluate(message.input, {});
    }
});
