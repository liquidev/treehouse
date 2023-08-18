use treehouse_format::{ast::Branch, pull::BranchKind};

use super::markdown;

pub fn branch_to_html(s: &mut String, branch: &Branch, source: &str) {
    s.push_str("<li>");
    {
        if !branch.children.is_empty() {
            s.push_str(match branch.kind {
                BranchKind::Expanded => "<details open>",
                BranchKind::Collapsed => "<details>",
            });
            s.push_str("<summary>");
        }

        let raw_block_content = &source[branch.content.clone()];
        let mut unindented_block_content = String::with_capacity(raw_block_content.len());
        let indent = " ".repeat(branch.indent_level);
        for line in raw_block_content.lines() {
            unindented_block_content.push_str(line.strip_prefix(&indent).unwrap_or(line));
            unindented_block_content.push('\n');
        }

        let markdown_parser = pulldown_cmark::Parser::new(&unindented_block_content);
        markdown::push_html(s, markdown_parser);

        if !branch.children.is_empty() {
            s.push_str("</summary>");
            branches_to_html(s, &branch.children, source);
            s.push_str("</details>");
        }
    }
    s.push_str("</li>");
}

pub fn branches_to_html(s: &mut String, branches: &[Branch], source: &str) {
    s.push_str("<ul>");
    for child in branches {
        branch_to_html(s, child, source);
    }
    s.push_str("</ul>");
}
