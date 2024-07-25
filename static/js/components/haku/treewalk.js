export const treewalk = {};
export const builtins = {};

treewalk.init = (input) => {
    return { input, scopes: [new Map(Object.entries(builtins))] };
};

treewalk.lookupVariable = (state, name) => {
    for (let i = state.scopes.length; i-- > 0; ) {
        let scope = state.scopes[i];
        if (scope.has(name)) {
            return scope.get(name);
        }
    }
    throw new Error(`variable ${name} is undefined`);
};

treewalk.eval = (state, node) => {
    switch (node.kind) {
        case "integer":
            let sourceString = state.input.substring(node.start, node.end);
            return parseInt(sourceString);

        case "identifier":
            return treewalk.lookupVariable(state, state.input.substring(node.start, node.end));

        case "list":
            let functionToCall = treewalk.eval(state, node.children[0]);
            return functionToCall(state, node);

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

export function makeFunction(state, paramNames, bodyExpr) {
    let capturedScopes = [];
    // Start from 1 to skip builtins, which are always present anyways.
    for (let i = 1; i < state.scopes.length; ++i) {
        // We don't really mutate the scopes after pushing them onto the stack, so keeping
        // references to them is okay.
        capturedScopes.push(state.scopes[i]);
    }

    return (state, node) => {
        if (node.children.length != paramNames.length + 1)
            throw new Error(
                `incorrect number of arguments: expected ${paramNames.length}, but got ${node.children.length - 1}`,
            );

        let scope = new Map();
        for (let i = 0; i < paramNames.length; ++i) {
            scope.set(paramNames[i], treewalk.eval(state, node.children[i + 1]));
        }

        state.scopes.push(...capturedScopes);
        state.scopes.push(scope);
        let result = treewalk.eval(state, bodyExpr);
        state.scopes.pop();

        return result;
    };
}

builtins.fn = (state, node) => {
    if (node.children.length != 3)
        throw new Error("an `fn` must have an argument list and a result expression");

    let params = node.children[1];
    if (node.children[1].kind != "list")
        throw new Error("expected parameter list as second argument to `fn`");

    let paramNames = [];
    for (let param of params.children) {
        if (param.kind != "identifier") {
            throw new Error("`fn` parameters must be identifiers");
        }
        paramNames.push(state.input.substring(param.start, param.end));
    }

    let expr = node.children[2];

    return makeFunction(state, paramNames, expr);
};
