use std::fmt::Write;

use pulldown_cmark::{BrokenLink, LinkType};
use treehouse_format::pull::BranchKind;

use crate::{
    config::Config,
    html::EscapeAttribute,
    state::{FileId, Treehouse},
    tree::{attributes::Content, mini_template, SemaBranchId},
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
    let mut class = String::from(class);
    if !branch.attributes.classes.branch.is_empty() {
        class.push(' ');
        class.push_str(&branch.attributes.classes.branch);
    }

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
            s.push_str("<summary class=\"branch-container\">");
        } else {
            s.push_str("<div class=\"branch-container\">");
        }

        s.push_str("<th-bp></th-bp>");

        let raw_block_content = &source.input()[branch.content.clone()];
        let mut final_markdown = String::with_capacity(raw_block_content.len());
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

            final_markdown.push_str(&line[space_count..]);
            final_markdown.push('\n');
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
                                    format!(
                                        "/b?{}",
                                        treehouse.tree.branch(branch_id).attributes.id
                                    )
                                    .into(),
                                    "".into(),
                                )
                            }),
                        "pic" => config.pics.get(linked).map(|filename| {
                            (format!("/static/pic/{}", &filename).into(), "".into())
                        }),
                        _ => None,
                    })
            } else {
                None
            }
        };
        if branch.attributes.template {
            final_markdown = mini_template::render(config, treehouse, &final_markdown);
        }
        let markdown_parser = pulldown_cmark::Parser::new_with_broken_link_callback(
            &final_markdown,
            {
                use pulldown_cmark::Options;
                Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES
            },
            Some(broken_link_callback),
        );
        s.push_str("<th-bc>");
        markdown::push_html(s, treehouse, config, markdown_parser);
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
        s.push_str("</th-bc>");

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
                "<a class=\"icon icon-permalink\" href=\"/b?{}\" title=\"permalink\"></a>",
                EscapeAttribute(&branch.attributes.id)
            )
            .unwrap();
        }
        s.push_str("</th-bb>");

        if has_children {
            s.push_str("</summary>");
            {
                s.push_str("<ul");
                if !branch.attributes.classes.branch_children.is_empty() {
                    write!(
                        s,
                        " class=\"{}\"",
                        EscapeAttribute(&branch.attributes.classes.branch_children)
                    )
                    .unwrap();
                }
                s.push('>');
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
