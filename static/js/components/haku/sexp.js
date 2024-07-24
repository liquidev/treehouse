export const lexer = {};

lexer.init = (input) => {
    return {
        input,
        position: 0,
    };
};

export const eof = "end of file";

lexer.current = (state) => {
    return state.position < state.input.length
        ? state.input.charAt(state.position)
        : eof;
};

lexer.advance = (state) => ++state.position;

lexer.advanceWhile = (state, fn) => {
    while (fn(lexer.current(state))) {
        lexer.advance(state);
    }
};

lexer.skipWhitespaceAndComments = (state) => {
    while (true) {
        let c = lexer.current(state);
        if (c == " " || c == "\t" || c == "\n" || c == "\r") {
            lexer.advance(state);
            continue;
        }
        if (c == ";") {
            while (
                lexer.current(state) != "\n" &&
                lexer.current(state) != eof
            ) {
                lexer.advance(state);
            }
            lexer.advance(state); // skip over newline, too
            continue;
        }

        break;
    }
};

export const isDigit = (c) => c >= "0" && c <= "9";
export const isIdentifier = (c) =>
    /^[a-zA-Z0-9+~!@$%^&*=<>+?/.,:\\|-]$/.test(c);

lexer.nextToken = (state) => {
    let c = lexer.current(state);

    if (isDigit(c)) {
        lexer.advanceWhile(state, isDigit);
        return "integer";
    }
    if (isIdentifier(c)) {
        lexer.advanceWhile(state, isIdentifier);
        return "identifier";
    }
    if (c == "(" || c == ")") {
        lexer.advance(state);
        return c;
    }
    if (c == eof) return eof;

    lexer.advance(state);
    return "error";
};

export function lex(input) {
    let tokens = [];

    let state = lexer.init(input);
    while (true) {
        lexer.skipWhitespaceAndComments(state);
        let start = state.position;
        let kind = lexer.nextToken(state);
        let end = state.position;
        tokens.push({ kind, start, end });
        if (kind == eof || kind == "error") break;
    }

    return tokens;
}

export const parser = {};

parser.init = (tokens) => {
    return {
        tokens,
        position: 0,
    };
};

parser.current = (state) => state.tokens[state.position];
parser.advance = (state) => {
    if (state.position < state.tokens.length - 1) {
        ++state.position;
    }
};

parser.parseExpr = (state) => {
    let token = parser.current(state);
    switch (token.kind) {
        case "integer":
        case "identifier":
            parser.advance(state);
            return { ...token };

        case "(":
            return parser.parseList(state, token);

        default:
            parser.advance(state);
            return {
                kind: "error",
                error: "unexpected token",
                start: token.start,
                end: token.end,
            };
    }
};

parser.parseList = (state, leftParen) => {
    parser.advance(state);

    let children = [];
    while (parser.current(state).kind != ")") {
        if (parser.current(state).kind == eof) {
            return {
                kind: "error",
                error: "missing closing parenthesis ')'",
                start: leftParen.start,
                end: leftParen.end,
            };
        }
        children.push(parser.parseExpr(state));
    }

    let rightParen = parser.current(state);
    parser.advance(state);

    return {
        kind: "list",
        children,
        start: leftParen.start,
        end: rightParen.end,
    };
};

parser.parseRoot = parser.parseExpr;

export function parse(input) {
    let state = parser.init(input);
    let expr = parser.parseRoot(state);

    if (parser.current(state).kind != eof) {
        let strayToken = parser.current(state);
        return {
            kind: "error",
            error: "found stray token after expression",
            start: strayToken.start,
            end: strayToken.end,
        };
    }

    return expr;
}

export function exprToString(expr, input) {
    let inputSubstring = input.substring(expr.start, expr.end);
    switch (expr.kind) {
        case "integer":
        case "identifier":
            return inputSubstring;

        case "list":
            return `(${expr.children.map((expr) => exprToString(expr, input)).join(" ")})`;

        case "error":
            return `<error ${expr.start}..${expr.end} '${inputSubstring}': ${expr.error}>`;
    }
}
