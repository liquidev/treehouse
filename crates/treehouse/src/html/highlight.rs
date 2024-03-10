//! Tokenizer and syntax highlighter inspired by the one found in rxi's lite.
//! I highly recommend checking it out!
//! https://github.com/rxi/lite/blob/master/data/core/tokenizer.lua
//! There's also a mirror of it in the JavaScript, used to power dynamically editable code blocks.
//!
//! Both of these syntax highlighters use the same JSON syntax definitions; however this one is
//! more limited, in that patterns do not support backtracking.
//! This is effectively enforced in the dynamic highlighter because this highlighter reports any
//! regex syntax errors upon site compilation.

pub mod compiled;
pub mod tokenize;

use std::{collections::HashMap, io};

use pulldown_cmark::escape::{escape_html, StrWrite};
use serde::{Deserialize, Serialize};

use self::compiled::CompiledSyntax;

/// Syntax definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Syntax {
    /// Patterns, matched sequentially (patterns at the beginning of the list take precedence.)
    pub patterns: Vec<Pattern>,

    /// Map of replacements to use if a pattern matches a string exactly.
    pub keywords: HashMap<String, Keyword>,
}

/// A pattern in a syntax definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pattern {
    /// Regular expression to match.
    pub regex: String,

    /// Flags to pass to the regex engine to alter how strings are matched.
    #[serde(default)]
    pub flags: Vec<RegexFlag>,

    /// Type to assign to the token. This can be any string, but only a select few have colors
    /// assigned.
    pub is: TokenTypes,
}

/// Assignable token types.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TokenTypes {
    /// Assign a single token type to the entire match.
    FullMatch(String),
    /// Assign individual token types to each capture.
    Captures(CaptureTokenTypes),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaptureTokenTypes {
    /// Token type to use outside captures.
    pub default: String,
    /// Token type to use inside captures.
    pub captures: Vec<String>,
}

/// Flag passed to the regex engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RegexFlag {
    /// Make `.` match line separators.
    DotMatchesNewline,
}

/// Keyword replacement.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Keyword {
    /// What to replace the token type with.
    pub into: String,

    /// Only replace the token type if it matches this one. If this is not present, any token type
    /// is replaced.
    pub only_replaces: Option<String>,
}

pub fn highlight(mut w: impl StrWrite, syntax: &CompiledSyntax, code: &str) -> io::Result<()> {
    let tokens = syntax.tokenize(code);
    for token in tokens {
        w.write_str("<span class=\"")?;
        escape_html(&mut w, &syntax.token_names[token.id])?;
        w.write_str("\">")?;
        escape_html(&mut w, &code[token.range])?;
        w.write_str("</span>")?;
    }
    Ok(())
}
