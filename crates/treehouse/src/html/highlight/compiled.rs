use std::collections::HashMap;

use regex::{Regex, RegexBuilder};
use tracing::error;

use super::{RegexFlag, Syntax, TokenTypes};

/// During compilation, token names are converted to numeric IDs for performance.
pub type TokenId = usize;

pub const TOKEN_ID_DEFAULT: TokenId = 0;

#[derive(Debug, Clone)]
pub struct CompiledSyntax {
    /// Lookup table which maps numeric IDs to token names.
    pub token_names: Vec<String>,

    pub patterns: Vec<CompiledPattern>,
    pub keywords: HashMap<String, CompiledKeyword>,
}

#[derive(Debug, Clone)]
pub enum CompiledTokenTypes {
    FullMatch(TokenId),
    Captures(CompiledCaptureTokenTypes),
}

#[derive(Debug, Clone)]
pub struct CompiledCaptureTokenTypes {
    pub default: TokenId,
    pub captures: Vec<TokenId>,
}

#[derive(Debug, Clone)]
pub struct CompiledPattern {
    pub regex: Regex,
    pub is: CompiledTokenTypes,
}

#[derive(Debug, Clone)]
pub struct CompiledKeyword {
    pub into: TokenId,
    pub only_replaces: Option<TokenId>,
}

pub fn compile_syntax(syntax: &Syntax) -> CompiledSyntax {
    let mut token_names = vec!["default".into()];
    let mut get_token_id = |name: &str| -> TokenId {
        if let Some(id) = token_names.iter().position(|n| n == name) {
            id
        } else {
            let id = token_names.len();
            token_names.push(name.to_owned());
            id
        }
    };

    let patterns = syntax
        .patterns
        .iter()
        .filter_map(|pattern| {
            // NOTE: `regex` has no support for sticky flags, so we need to anchor the match to the
            // start ourselves.
            let regex = RegexBuilder::new(&format!(
                "^{}",
                // If there's an existing `^`, it should not cause compilation errors for the user.
                pattern.regex.strip_prefix('^').unwrap_or(&pattern.regex)
            ))
            .dot_matches_new_line(pattern.flags.contains(&RegexFlag::DotMatchesNewline))
            .build()
            .map_err(|e| {
                // NOTE: This could probably use better diagnostics, but it's pretty much
                // impossible to get a source span out of serde's output (because it forgoes
                // source information, rightfully so.) Therefore we have to settle on
                // a poor man's error log.
                error!("regex compilation error in pattern {pattern:?}: {e}");
            })
            .ok()?;
            Some(CompiledPattern {
                regex,
                is: match &pattern.is {
                    TokenTypes::FullMatch(name) => {
                        CompiledTokenTypes::FullMatch(get_token_id(name))
                    }
                    TokenTypes::Captures(types) => {
                        CompiledTokenTypes::Captures(CompiledCaptureTokenTypes {
                            default: get_token_id(&types.default),
                            captures: types
                                .captures
                                .iter()
                                .map(|name| get_token_id(name))
                                .collect(),
                        })
                    }
                },
            })
        })
        .collect();
    let keywords = syntax
        .keywords
        .iter()
        .map(|(text, keyword)| {
            (
                text.clone(),
                CompiledKeyword {
                    into: get_token_id(&keyword.into),
                    only_replaces: keyword.only_replaces.as_deref().map(&mut get_token_id),
                },
            )
        })
        .collect();

    CompiledSyntax {
        token_names,
        patterns,
        keywords,
    }
}
