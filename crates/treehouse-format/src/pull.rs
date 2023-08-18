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

impl<'a> Parser<'a> {
    fn current(&self) -> Option<char> {
        self.input[self.position..].chars().next()
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

    fn eat_until(&mut self, cond: impl Fn(char) -> bool) {
        while self.current().map(&cond).is_some_and(|x| !x) {
            self.advance();
        }
        self.advance();
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
    ) -> Result<(), ParseError> {
        loop {
            self.eat_until(|c| c == '\n');
            let before_indentation = self.position;
            let line_indent_level = self.eat_as_long_as(' ');
            let after_indentation = self.position;
            if self.current().map(&cond).is_some_and(identity) || self.current().is_none() {
                self.position = before_indentation;
                break;
            } else if !matches!(self.current(), Some('\n')) && line_indent_level < indent_level {
                return Err(ParseErrorKind::InconsistentIndentation {
                    got: line_indent_level,
                    expected: indent_level,
                }
                .at(before_indentation..after_indentation));
            }
        }
        Ok(())
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
            self.eat_indented_lines_until(indent_level, |c| c == '-' || c == '+')?;
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
        self.eat_indented_lines_until(indent_level, |c| c == '-' || c == '+' || c == '%')?;
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
