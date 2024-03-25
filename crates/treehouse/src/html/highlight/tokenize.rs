use std::ops::Range;

use super::compiled::{CompiledSyntax, CompiledTokenTypes, TokenId, TOKEN_ID_DEFAULT};

pub struct Token {
    pub id: TokenId,
    pub range: Range<usize>,
}

impl CompiledSyntax {
    pub fn tokenize(&self, text: &str) -> Vec<Token> {
        let mut tokens = vec![];

        let mut i = 0;
        while i < text.len() {
            let mut had_match = false;
            for pattern in &self.patterns {
                match &pattern.is {
                    CompiledTokenTypes::FullMatch(id) => {
                        if let Some(regex_match) = pattern.regex.find(&text[i..]) {
                            push_token(&mut tokens, *id, i..i + regex_match.range().end);
                            i += regex_match.range().end;
                            had_match = true;
                            break;
                        }
                    }
                    CompiledTokenTypes::Captures(types) => {
                        if let Some(captures) = pattern.regex.captures(&text[i..]) {
                            let whole_match = captures.get(0).unwrap();
                            let mut last_match_end = 0;
                            for (index, capture) in captures
                                .iter()
                                .skip(1)
                                .enumerate()
                                .filter_map(|(i, m)| m.map(|m| (i, m)))
                            {
                                let id = types
                                    .captures
                                    .get(index)
                                    .copied()
                                    .unwrap_or(TOKEN_ID_DEFAULT);
                                push_token(
                                    &mut tokens,
                                    types.default,
                                    i + last_match_end..i + capture.range().start,
                                );
                                push_token(
                                    &mut tokens,
                                    id,
                                    i + capture.range().start..i + capture.range().end,
                                );
                                last_match_end = capture.range().end;
                            }
                            push_token(
                                &mut tokens,
                                types.default,
                                i + last_match_end..i + whole_match.range().end,
                            );
                            i += whole_match.range().end;
                            had_match = true;
                            break;
                        }
                    }
                }
            }

            if !had_match {
                push_token(&mut tokens, TOKEN_ID_DEFAULT, i..i + 1);
                i += 1;
            }
        }

        for token in &mut tokens {
            if let Some(keyword) = self.keywords.get(&text[token.range.clone()]) {
                if keyword.only_replaces.is_none() || Some(token.id) == keyword.only_replaces {
                    token.id = keyword.into;
                }
            }
        }

        tokens
    }
}

fn push_token(tokens: &mut Vec<Token>, id: TokenId, range: Range<usize>) {
    if range.is_empty() {
        return;
    }

    if let Some(previous_token) = tokens.last_mut() {
        if previous_token.id == id {
            previous_token.range.end = range.end;
            return;
        }
    }
    tokens.push(Token { id, range });
}
