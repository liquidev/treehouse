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
                    CompiledTokenTypes::Captures(types) => { /* TODO */ }
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
    if let Some(previous_token) = tokens.last_mut() {
        if previous_token.id == id {
            previous_token.range.end = range.end;
            return;
        }
    }
    tokens.push(Token { id, range });
}
