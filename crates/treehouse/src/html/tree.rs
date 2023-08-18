use treehouse_format::{ast::Branch, pull::BranchKind};

use super::markdown;

pub fn branch_to_html(s: &mut String, branch: &Branch, source: &str) {
    s.push_str(if !branch.children.is_empty() {
        "<li class=\"branch\">"
    } else {
        "<li class=\"leaf\">"
    });
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
        for line in raw_block_content.lines() {
            // Bit of a jank way to remove at most branch.indent_level spaces from the front.
            let mut space_count = 0;
            for i in 0..branch.indent_level {
                if line.as_bytes().get(i).copied() == Some(b' ') {
                    space_count += 1;
                } else {
                    break;
                }
            }

            unindented_block_content.push_str(&line[space_count..]);
            unindented_block_content.push('\n');
        }

        let markdown_parser = pulldown_cmark::Parser::new_ext(&unindented_block_content, {
            use pulldown_cmark::Options;
            Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES
        });
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
