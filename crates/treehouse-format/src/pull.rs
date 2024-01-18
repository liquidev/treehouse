use std::{convert::identity, ops::Range};

use crate::{ParseError, ParseErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchKind {
    /// Expanded by default.
    Expanded,
    /// Folded by default.
    Collapsed,
}

impl BranchKind {
    pub fn char(&self) -> char {
        match self {
            BranchKind::Expanded => '-',
            BranchKind::Collapsed => '+',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchEvent {
    pub indent_level: usize,
    pub kind: BranchKind,
    pub kind_span: Range<usize>,
    pub content: Range<usize>,
    pub attributes: Option<Attributes>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attributes {
    pub percent: Range<usize>,
    pub data: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser<'a> {
    pub input: &'a str,
    pub position: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AllowCodeBlocks {
    No,
    Yes,
}

impl<'a> Parser<'a> {
    fn current(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn current_starts_with(&self, s: &str) -> bool {
        self.input[self.position..].starts_with(s)
    }

    fn advance(&mut self) {
        self.position += self.current().map(|c| c.len_utf8()).unwrap_or(0);
    }

    fn eat_as_long_as(&mut self, c: char) -> usize {
        let mut count = 0;
        while self.current() == Some(c) {
            count += 1;
            self.advance();
        }
        count
    }

    fn eat_while(&mut self, cond: impl Fn(char) -> bool) {
        while self.current().map(&cond).is_some_and(|x| x) {
            self.advance();
        }
    }

    fn eat_until_line_break(&mut self) {
        loop {
            match self.current() {
                Some('\r') => {
                    self.advance();
                    if self.current() == Some('\n') {
                        self.advance();
                        break;
                    }
                }
                Some('\n') => {
                    self.advance();
                    break;
                }
                Some(_) => self.advance(),
                None => break,
            }
        }
    }

    pub fn peek_indent_level(&mut self) -> usize {
        let position = self.position;
        let indent_level = self.eat_as_long_as(' ');
        self.position = position;
        indent_level
    }

    fn eat_indented_lines_until(
        &mut self,
        indent_level: usize,
        cond: impl Fn(char) -> bool,
        allow_code_blocks: AllowCodeBlocks,
    ) -> Result<(), ParseError> {
        let mut code_block: Option<Range<usize>> = None;
        loop {
            if let Some(range) = &code_block {
                self.eat_while(|c| c == ' ');
                if self.current_starts_with("```") {
                    code_block = None;
                    self.position += 3;
                    self.eat_until_line_break();
                    continue;
                }
                self.eat_until_line_break();

                if self.current().is_none() {
                    return Err(ParseErrorKind::UnterminatedCodeBlock.at(range.clone()));
                }
            } else {
                self.eat_while(|c| c == ' ');
                if allow_code_blocks == AllowCodeBlocks::Yes && self.current_starts_with("```") {
                    code_block = Some(self.position..self.position + 3);
                    self.position += 3;
                    continue;
                }

                self.eat_until_line_break();
                let before_indentation = self.position;
                let line_indent_level = self.eat_as_long_as(' ');
                let after_indentation = self.position;
                if self.current().map(&cond).is_some_and(identity) || self.current().is_none() {
                    self.position = before_indentation;
                    break;
                } else if !matches!(self.current(), Some('\n') | Some('\r'))
                    && line_indent_level < indent_level
                {
                    return Err(ParseErrorKind::InconsistentIndentation {
                        got: line_indent_level,
                        expected: indent_level,
                    }
                    .at(before_indentation..after_indentation));
                }
            }
        }
        Ok(())
    }

    pub fn top_level_attributes(&mut self) -> Result<Option<Attributes>, ParseError> {
        let start = self.position;
        match self.current() {
            Some('%') => {
                let after_one_percent = self.position;
                self.advance();
                if self.current() == Some('%') {
                    self.advance();
                    let after_two_percent = self.position;
                    self.eat_indented_lines_until(
                        0,
                        |c| c == '-' || c == '+' || c == '%',
                        AllowCodeBlocks::No,
                    )?;
                    let end = self.position;
                    Ok(Some(Attributes {
                        percent: start..after_two_percent,
                        data: after_two_percent..end,
                    }))
                } else {
                    self.position = after_one_percent;
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub fn next_branch(&mut self) -> Result<Option<BranchEvent>, ParseError> {
        if self.current().is_none() {
            return Ok(None);
        }

        let indent_level = self.eat_as_long_as(' ');

        let attributes = if self.current() == Some('%') {
            let start = self.position;
            self.advance();
            let after_percent = self.position;
            self.eat_indented_lines_until(
                indent_level,
                |c| c == '-' || c == '+',
                AllowCodeBlocks::No,
            )?;
            self.eat_as_long_as(' ');
            let end = self.position;
            Some(Attributes {
                percent: start..after_percent,
                data: after_percent..end,
            })
        } else {
            None
        };

        let kind_start = self.position;
        let kind = match self.current() {
            Some('-') => BranchKind::Expanded,
            Some('+') => BranchKind::Collapsed,
            _ => return Err(ParseErrorKind::BranchKindExpected.at(kind_start..kind_start + 1)),
        };
        self.advance();
        let kind_end = self.position;

        let content_start = self.position;
        self.eat_indented_lines_until(
            indent_level,
            |c| c == '-' || c == '+' || c == '%',
            AllowCodeBlocks::Yes,
        )?;
        let content_end = self.position;

        Ok(Some(BranchEvent {
            indent_level,
            attributes,
            kind,
            kind_span: kind_start..kind_end,
            content: content_start..content_end,
        }))
    }
}
