console = {
    log(...message) {
        postMessage({
            kind: "output",
            output: {
                kind: "log",
                message: [...message],
            }
        });
    }
};

addEventListener("message", event => {
    let message = event.data;
    if (message.action == "eval") {
        try {
            let func = new Function(message.input);
            let result = func.apply({});
            postMessage({
                kind: "output",
                output: {
                    kind: "result",
                    message: [result],
                }
            });
        } catch (error) {
            postMessage({
                kind: "output",
                output: {
                    kind: "error",
                    message: [error.toString()],
                }
            });
        }

        postMessage({
            kind: "evalComplete",
        });
    }
});
