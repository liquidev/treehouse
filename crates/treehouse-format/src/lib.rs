use std::ops::Range;

pub mod ast;
pub mod pull;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    #[error("branch kind (`+` or `-`) expected")]
    BranchKindExpected,

    #[error("root branches must not be indented")]
    RootIndentLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{range:?}: {kind}")]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub range: Range<usize>,
}

impl ParseErrorKind {
    pub fn at(self, range: Range<usize>) -> ParseError {
        ParseError { kind: self, range }
    }
}
