export const treewalk = {};
export const builtins = {};

treewalk.init = (env, input) => {
    return {
        input,
        scopes: [new Map(Object.entries(builtins)), env],
        env,
    };
};

treewalk.lookupVariable = (state, name) => {
    for (let i = state.scopes.length; i-- > 0; ) {
        let scope = state.scopes[i];
        if (scope.has(name)) {
            return scope.get(name);
        }
    }
    console.log(new Error().stack);
    throw new Error(`variable ${name} is undefined`);
};

treewalk.eval = (state, node) => {
    switch (node.kind) {
        case "integer":
            return parseInt(node.source);

        case "identifier":
            return treewalk.lookupVariable(state, node.source);

        case "list":
            let functionToCall = treewalk.eval(state, node.children[0]);
            return functionToCall(state, node);

        case "toplevel":
            let result = undefined;
            for (let i = 0; i < node.children.length; ++i) {
                result = treewalk.eval(state, node.children[i]);
                if (result !== undefined && i != node.children.length - 1)
                    throw new Error(`expression ${i + 1} had a result despite not being the last`);
            }
            return result;

        default:
            throw new Error(`unhandled node kind: ${node.kind}`);
    }
};

export function run(env, input, node) {
    let state = treewalk.init(env, input);
    return treewalk.eval(state, node);
}

function arithmeticBuiltin(op) {
    return (state, node) => {
        if (node.children.length < 3)
            throw new Error("arithmetic operations require at least two arguments");

        let result = treewalk.eval(state, node.children[1]);
        for (let i = 2; i < node.children.length; ++i) {
            result = op(result, treewalk.eval(state, node.children[i]));
        }
        return result;
    };
}

function comparisonBuiltin(op) {
    return (state, node) => {
        if (node.children.length != 3)
            throw new Error("comparison operators require exactly two arguments");

        let a = treewalk.eval(state, node.children[1]);
        let b = treewalk.eval(state, node.children[2]);
        return op(a, b) ? 1 : 0;
    };
}

builtins["+"] = arithmeticBuiltin((a, b) => a + b);
builtins["-"] = arithmeticBuiltin((a, b) => a - b);
builtins["*"] = arithmeticBuiltin((a, b) => a * b);
builtins["/"] = arithmeticBuiltin((a, b) => a / b);

builtins["="] = comparisonBuiltin((a, b) => a === b);
builtins["<"] = comparisonBuiltin((a, b) => a < b);

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
        paramNames.push(param.source);
    }

    let expr = node.children[2];

    return makeFunction(state, paramNames, expr);
};

builtins["if"] = (state, node) => {
    if (node.children.length != 4)
        throw new Error("an `if` must have a condition, true expression, and false expression");

    let condition = treewalk.eval(state, node.children[1]);
    if (condition !== 0) {
        return treewalk.eval(state, node.children[2]);
    } else {
        return treewalk.eval(state, node.children[3]);
    }
};

builtins.def = (state, node) => {
    if (node.children.length != 3)
        throw new Error(
            "a `def` expects the name of the variable to assign, and the value to assign to the variable",
        );

    if (node.children[1].kind != "identifier")
        throw new Error("variable name must be an identifier");

    let name = node.children[1];
    let value = treewalk.eval(state, node.children[2]);
    state.env.set(name.source, value);
};
