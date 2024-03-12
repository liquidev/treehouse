// This tokenizer is highly inspired by the one found in rxi's lite.
// I highly recommend checking it out!
// https://github.com/rxi/lite/blob/master/data/core/tokenizer.lua
// There's also a mirror of it in the static generator, to enable highlighting of code blocks which
// are *not* JavaScript-powered.

export function compileSyntax(def) {
    for (let pattern of def.patterns) {
        let flags = "dy";
        if (pattern.flags != null) {
            if ("dotMatchesNewline" in pattern.flags) {
                flags += "s";
            }
        }
        pattern.regex = new RegExp(pattern.regex, flags);
    }
    def.keywords = new Map(Object.entries(def.keywords));
    return def;
}

function pushToken(tokens, kind, string) {
    let previousToken = tokens[tokens.length - 1];
    if (previousToken != null && previousToken.kind == kind) {
        previousToken.string += string;
    } else {
        tokens.push({ kind, string });
    }
}

function tokenize(text, syntax) {
    let tokens = [];
    let i = 0;

    while (i < text.length) {
        let hadMatch = false;
        for (let pattern of syntax.patterns) {
            let match;
            pattern.regex.lastIndex = i;
            if ((match = pattern.regex.exec(text)) != null) {
                if (typeof pattern.is == "object") {
                    let lastMatchEnd = i;
                    for (let i = 1; i < match.indices.length; ++i) {
                        let [start, end] = match.indices[i];
                        if (match.indices[i] != null) {
                            pushToken(tokens, pattern.is.default, text.substring(lastMatchEnd, start));
                            pushToken(tokens, pattern.is.captures[i], text.substring(start, end));
                        }
                    }
                } else {
                    pushToken(tokens, pattern.is, match[0]);
                }
                i = pattern.regex.lastIndex;
                hadMatch = true;
                break;
            }
        }

        // Base case: no pattern matched, just add the current character to the output.
        if (!hadMatch) {
            pushToken(tokens, "default", text.substring(i, i + 1));
            ++i;
        }
    }

    for (let token of tokens) {
        let replacement = syntax.keywords.get(token.string);
        if (replacement != null) {
            if (replacement.onlyReplaces == null || token.kind == replacement.onlyReplaces) {
                token.kind = replacement.into;
            }
        }
    }

    return tokens;
}

export function highlight(element, syntax, customize = null) {
    let tokens = tokenize(element.textContent, syntax);

    element.textContent = "";
    element.classList.add("th-syntax-highlighting");
    for (let token of tokens) {
        let span = document.createElement("span");
        span.textContent = token.string;
        span.classList.add(token.kind);
        if (customize != null) {
            customize(token, span);
        }
        element.appendChild(span);
    }
}
