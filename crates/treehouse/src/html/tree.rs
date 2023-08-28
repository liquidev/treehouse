use std::fmt::Write;

use pulldown_cmark::{BrokenLink, LinkType};
use treehouse_format::pull::BranchKind;

use crate::{
    config::Config,
    html::EscapeAttribute,
    state::{FileId, Treehouse},
    tree::{attributes::Content, SemaBranchId},
};

use super::{markdown, EscapeHtml};

pub fn branch_to_html(
    s: &mut String,
    treehouse: &mut Treehouse,
    config: &Config,
    file_id: FileId,
    branch_id: SemaBranchId,
) {
    let source = treehouse.source(file_id);
    let branch = treehouse.tree.branch(branch_id);

    let has_children =
        !branch.children.is_empty() || matches!(branch.attributes.content, Content::Link(_));

    let class = if has_children { "branch" } else { "leaf" };
    let component = if let Content::Link(_) = branch.attributes.content {
        "th-b-linked"
    } else {
        "th-b"
    };

    let linked_branch = if let Content::Link(link) = &branch.attributes.content {
        format!(" data-th-link=\"{}\"", EscapeHtml(link))
    } else {
        String::new()
    };

    let do_not_persist = if branch.attributes.do_not_persist {
        " data-th-do-not-persist=\"\""
    } else {
        ""
    };

    write!(
        s,
        "<li is=\"{component}\" class=\"{class}\" id=\"{}\"{linked_branch}{do_not_persist}>",
        EscapeAttribute(&branch.html_id)
    )
    .unwrap();
    {
        if has_children {
            s.push_str(match branch.kind {
                BranchKind::Expanded => "<details open>",
                BranchKind::Collapsed => "<details>",
            });
            s.push_str("<summary>");
        } else {
            s.push_str("<div>");
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

        let broken_link_callback = &mut |broken_link: BrokenLink<'_>| {
            if let LinkType::Reference | LinkType::Shortcut = broken_link.link_type {
                broken_link
                    .reference
                    .split_once(':')
                    .and_then(|(kind, linked)| match kind {
                        "def" => config
                            .defs
                            .get(linked)
                            .map(|link| (link.clone().into(), "".into())),
                        "branch" => treehouse
                            .branches_by_named_id
                            .get(linked)
                            .map(|&branch_id| {
                                (
                                    format!("#{}", treehouse.tree.branch(branch_id).html_id).into(),
                                    "".into(),
                                )
                            }),
                        _ => None,
                    })
            } else {
                None
            }
        };
        let markdown_parser = pulldown_cmark::Parser::new_with_broken_link_callback(
            &unindented_block_content,
            {
                use pulldown_cmark::Options;
                Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES
            },
            Some(broken_link_callback),
        );
        if has_children {
            s.push_str("<span class=\"branch-summary\">")
        }
        markdown::push_html(s, treehouse, config, markdown_parser);
        if has_children {
            s.push_str("</span>")
        }

        if let Content::Link(link) = &branch.attributes.content {
            write!(
                s,
                "<noscript><a class=\"navigate icon-go\" href=\"{}/{}.html\">Go to linked tree: <code>{}</code></a></noscript>",
                EscapeAttribute(&config.site),
                EscapeAttribute(link),
                EscapeHtml(link),
            )
            .unwrap();
        }

        s.push_str("<th-bb>");
        {
            if let Content::Link(link) = &branch.attributes.content {
                write!(
                    s,
                    "<a class=\"icon icon-go\" href=\"{}/{}.html\" title=\"linked tree\"></a>",
                    EscapeAttribute(&config.site),
                    EscapeAttribute(link),
                )
                .unwrap();
            }

            write!(
                s,
                "<a class=\"icon icon-permalink\" href=\"#{}\" title=\"permalink\"></a>",
                EscapeAttribute(&branch.html_id)
            )
            .unwrap();
        }
        s.push_str("</th-bb>");

        if has_children {
            s.push_str("</summary>");
            {
                s.push_str("<ul>");
                let num_children = branch.children.len();
                for i in 0..num_children {
                    let child_id = treehouse.tree.branch(branch_id).children[i];
                    branch_to_html(s, treehouse, config, file_id, child_id);
                }
                s.push_str("</ul>");
            }
            s.push_str("</details>");
        } else {
            s.push_str("</div>");
        }
    }
    s.push_str("</li>");
}

pub fn branches_to_html(
    s: &mut String,
    treehouse: &mut Treehouse,
    config: &Config,
    file_id: FileId,
    branches: &[SemaBranchId],
) {
    s.push_str("<ul>");
    for &child in branches {
        branch_to_html(s, treehouse, config, file_id, child);
    }
    s.push_str("</ul>");
}
