use std::ops::Range;

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
    pub attributes: Range<usize>,
    pub kind: BranchKind,
    pub kind_span: Range<usize>,
    pub content: Range<usize>,
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

    fn eat_until(&mut self, c: char) {
        while self.current() != Some(c) {
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

    pub fn next_branch(&mut self) -> Result<Option<BranchEvent>, ParseError> {
        if self.current().is_none() {
            return Ok(None);
        }

        let indent_level = self.eat_as_long_as(' ');

        // TODO: Configs
        let config_start = self.position;
        let config_end = self.position;

        let kind_start = self.position;
        let kind = match self.current() {
            Some('-') => BranchKind::Expanded,
            Some('+') => BranchKind::Collapsed,
            _ => return Err(ParseErrorKind::BranchKindExpected.at(kind_start..kind_start + 1)),
        };
        self.advance();
        let kind_end = self.position;

        let content_start = self.position;
        loop {
            self.eat_until('\n');
            if let Some('\n') | None = self.current() {
                self.advance();
                break;
            }
        }
        let content_end = self.position;

        Ok(Some(BranchEvent {
            indent_level,
            attributes: config_start..config_end,
            kind,
            kind_span: kind_start..kind_end,
            content: content_start..content_end,
        }))
    }
}
