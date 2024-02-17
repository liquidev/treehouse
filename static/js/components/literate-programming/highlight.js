// This tokenizer is highly inspired by the one found in rxi's lite.
// I highly recommend checking it out!
// https://github.com/rxi/lite/blob/master/data/core/tokenizer.lua

export function compileSyntax(def) {
    for (let pattern of def.patterns) {
        // Remove g (global) flag as it would interfere with the lexis process. We only want to match
        // the first token at the cursor.
        let flags = pattern.regex.flags.replace("g", "");
        // Add d (indices) and y (sticky) flags so that we can tell where the matches start and end.
        pattern.regex = new RegExp(pattern.regex, "y" + flags);
    }
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
                pushToken(tokens, pattern.as, match[0]); // TODO
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
