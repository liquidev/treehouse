//! Minimalistic templating engine that integrates with the .tree format and Markdown.
//!
//! Mostly to avoid pulling in Handlebars everywhere; mini_template, unlike Handlebars, also allows
//! for injecting *custom, stateful* context into the renderer, which is important for things like
//! the `pic` template to work.

use std::ops::Range;

use pulldown_cmark::escape::escape_html;

use crate::{config::Config, state::Treehouse};

struct Lexer<'a> {
    input: &'a str,
    position: usize,

    // Despite this parser's intentional simplicity, a peekahead buffer needs to be used for
    // performance because tokens are usually quite long and therefore reparsing them would be
    // too expensive.
    peek_buffer: Option<(Token, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    /// Verbatim text, may be inside of a template.
    Text,
    Open(EscapingMode), // {%
    Close,              // %}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscapingMode {
    EscapeHtml,
    NoEscaping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    range: Range<usize>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            peek_buffer: None,
        }
    }

    fn current(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance(&mut self) {
        self.position += self.current().map(|c| c.len_utf8()).unwrap_or(0);
    }

    fn create_token(&self, start: usize, kind: TokenKind) -> Token {
        Token {
            kind,
            range: start..self.position,
        }
    }

    fn next_inner(&mut self) -> Option<Token> {
        if let Some((token, after_token)) = self.peek_buffer.take() {
            self.position = after_token;
            return Some(token);
        }

        let start = self.position;
        match self.current() {
            Some('{') => {
                self.advance();
                if self.current() == Some('%') {
                    self.advance();
                    if self.current() == Some('!') {
                        Some(self.create_token(start, TokenKind::Open(EscapingMode::NoEscaping)))
                    } else {
                        Some(self.create_token(start, TokenKind::Open(EscapingMode::EscapeHtml)))
                    }
                } else {
                    self.advance();
                    Some(self.create_token(start, TokenKind::Text))
                }
            }
            Some('%') => {
                self.advance();
                if self.current() == Some('}') {
                    self.advance();
                    Some(self.create_token(start, TokenKind::Close))
                } else {
                    self.advance();
                    Some(self.create_token(start, TokenKind::Text))
                }
            }
            Some(_) => {
                while !matches!(self.current(), Some('{' | '%') | None) {
                    self.advance();
                }
                Some(self.create_token(start, TokenKind::Text))
            }
            None => None,
        }
    }

    fn peek_inner(&mut self) -> Option<Token> {
        let position = self.position;
        let token = self.next();
        let after_token = self.position;
        self.position = position;

        if let Some(token) = token.clone() {
            self.peek_buffer = Some((token, after_token));
        }

        token
    }

    fn next(&mut self) -> Option<Token> {
        self.next_inner().map(|mut token| {
            // Coalesce multiple Text tokens into one.
            if token.kind == TokenKind::Text {
                while let Some(Token {
                    kind: TokenKind::Text,
                    ..
                }) = self.peek_inner()
                {
                    let next_token = self.next_inner().unwrap();
                    token.range.end = next_token.range.end;
                }
            }
            token
        })
    }
}

struct Renderer<'a> {
    lexer: Lexer<'a>,
    output: String,
}

struct InvalidTemplate;

impl<'a> Renderer<'a> {
    fn emit_token_verbatim(&mut self, token: &Token) {
        self.output.push_str(&self.lexer.input[token.range.clone()]);
    }

    fn render(&mut self, config: &Config, treehouse: &Treehouse) {
        let kind_of = |token: &Token| token.kind;

        while let Some(token) = self.lexer.next() {
            match token.kind {
                TokenKind::Open(escaping) => {
                    let inside = self.lexer.next();
                    let close = self.lexer.next();

                    if let Some((TokenKind::Text, TokenKind::Close)) = inside
                        .as_ref()
                        .map(kind_of)
                        .zip(close.as_ref().map(kind_of))
                    {
                        match Self::render_template(
                            config,
                            treehouse,
                            self.lexer.input[inside.as_ref().unwrap().range.clone()].trim(),
                        ) {
                            Ok(s) => match escaping {
                                EscapingMode::EscapeHtml => {
                                    _ = escape_html(&mut self.output, &s);
                                }
                                EscapingMode::NoEscaping => self.output.push_str(&s),
                            },
                            Err(InvalidTemplate) => {
                                inside.inspect(|token| self.emit_token_verbatim(token));
                                close.inspect(|token| self.emit_token_verbatim(token));
                            }
                        }
                    } else {
                        inside.inspect(|token| self.emit_token_verbatim(token));
                        close.inspect(|token| self.emit_token_verbatim(token));
                    }
                }
                _ => self.emit_token_verbatim(&token),
            }
        }
    }

    fn render_template(
        config: &Config,
        _treehouse: &Treehouse,
        template: &str,
    ) -> Result<String, InvalidTemplate> {
        let (function, arguments) = template.split_once(' ').unwrap_or((template, ""));
        match function {
            "pic" => Ok(config.pic_url(arguments)),
            "c++" => Ok("<script>alert(1)</script>".into()),
            _ => Err(InvalidTemplate),
        }
    }
}

pub fn render(config: &Config, treehouse: &Treehouse, input: &str) -> String {
    let mut renderer = Renderer {
        lexer: Lexer::new(input),
        output: String::new(),
    };
    renderer.render(config, treehouse);
    renderer.output
}
