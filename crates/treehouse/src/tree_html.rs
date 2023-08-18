use treehouse_format::{ast::Branch, pull::BranchKind};

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
        s.push_str(&source[branch.content.clone()]);
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
