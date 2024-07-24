export const treewalk = {};
export const builtins = {};

treewalk.init = (input) => {
    return { input };
};

treewalk.eval = (state, node) => {
    switch (node.kind) {
        case "integer":
            let sourceString = state.input.substring(node.start, node.end);
            return parseInt(sourceString);

        case "list":
            let functionToCall = node.children[0];
            let builtin = builtins[state.input.substring(functionToCall.start, functionToCall.end)];
            return builtin(state, node);

        default:
            throw new Error(`unhandled node kind: ${node.kind}`);
    }
};

export function run(input, node) {
    let state = treewalk.init(input);
    return treewalk.eval(state, node);
}

function arithmeticBuiltin(op) {
    return (state, node) => {
        let result = treewalk.eval(state, node.children[1]);
        for (let i = 2; i < node.children.length; ++i) {
            result = op(result, treewalk.eval(state, node.children[i]));
        }
        return result;
    };
}

builtins["+"] = arithmeticBuiltin((a, b) => a + b);
builtins["-"] = arithmeticBuiltin((a, b) => a - b);
builtins["*"] = arithmeticBuiltin((a, b) => a * b);
builtins["/"] = arithmeticBuiltin((a, b) => a / b);
