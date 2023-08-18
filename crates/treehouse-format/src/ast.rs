use std::ops::Range;

use crate::{
    pull::{BranchEvent, BranchKind, Parser},
    ParseError, ParseErrorKind,
};

#[derive(Debug, Clone)]
pub struct Roots {
    pub branches: Vec<Branch>,
}

impl Roots {
    pub fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut branches = vec![];
        while let Some((branch, indent_level)) = Branch::parse_with_indent_level(parser)? {
            if indent_level != 0 {
                return Err(ParseErrorKind::RootIndentLevel.at(branch.kind_span));
            }
            branches.push(branch);
        }
        Ok(Self { branches })
    }
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub attributes: Range<usize>,
    pub kind: BranchKind,
    pub kind_span: Range<usize>,
    pub content: Range<usize>,
    pub children: Vec<Branch>,
}

impl From<BranchEvent> for Branch {
    fn from(branch: BranchEvent) -> Self {
        Self {
            attributes: branch.attributes,
            kind: branch.kind,
            kind_span: branch.kind_span,
            content: branch.content,
            children: vec![],
        }
    }
}

impl Branch {
    pub fn parse_with_indent_level(
        parser: &mut Parser,
    ) -> Result<Option<(Self, usize)>, ParseError> {
        if let Some(branch_event) = parser.next_branch()? {
            let own_indent_level = branch_event.indent_level;
            let mut branch = Branch::from(branch_event);
            let children_indent_level = parser.peek_indent_level();
            if children_indent_level > own_indent_level {
                while parser.peek_indent_level() == children_indent_level {
                    if let Some(child) = Branch::parse(parser)? {
                        branch.children.push(child);
                    } else {
                        break;
                    }
                }
            }
            Ok(Some((branch, own_indent_level)))
        } else {
            Ok(None)
        }
    }

    pub fn parse(parser: &mut Parser) -> Result<Option<Self>, ParseError> {
        Ok(Self::parse_with_indent_level(parser)?.map(|(branch, _)| branch))
    }
}
