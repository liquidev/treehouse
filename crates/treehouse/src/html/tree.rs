use std::fmt::Write;

use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle, Severity};
use treehouse_format::{ast::Branch, pull::BranchKind};

use crate::{
    html::EscapeAttribute,
    state::{toml_error_to_diagnostic, FileId, TomlError, Treehouse},
};

use super::{
    attributes::{Attributes, Content},
    markdown, EscapeHtml,
};

pub fn branch_to_html(s: &mut String, treehouse: &mut Treehouse, file_id: FileId, branch: &Branch) {
    let attributes = parse_attributes(treehouse, file_id, branch);

    // Reborrow because the closure requires unique access (it adds a new diagnostic.)
    let source = treehouse.get_source(file_id);

    let has_children =
        !branch.children.is_empty() || matches!(attributes.content, Content::Link(_));

    let class = if has_children { "branch" } else { "leaf" };
    let linked_branch = if let Content::Link(link) = &attributes.content {
        format!(
            " is=\"th-linked-branch\" data-th-link=\"{}\"",
            EscapeHtml(link)
        )
    } else {
        String::new()
    };
    write!(
        s,
        "<li class=\"{class}\" id=\"{}\"{linked_branch}>",
        EscapeAttribute(&attributes.id)
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

        let markdown_parser = pulldown_cmark::Parser::new_ext(&unindented_block_content, {
            use pulldown_cmark::Options;
            Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES
        });
        markdown::push_html(s, markdown_parser);

        if let Content::Link(link) = &attributes.content {
            write!(
                s,
                "<noscript><a class=\"navigate icon-go\" href=\"{}.html\">Go to linked tree: <code>{}</code></a></noscript>",
                EscapeAttribute(link),
                EscapeHtml(link),
            )
            .unwrap();
        }

        s.push_str("<th-bb>");
        {
            if let Content::Link(link) = &attributes.content {
                write!(
                    s,
                    "<a class=\"icon icon-go\" href=\"{}.html\" title=\"linked tree\"></a>",
                    EscapeAttribute(link),
                )
                .unwrap();
            }

            write!(
                s,
                "<a class=\"icon icon-permalink\" href=\"#{}\" title=\"permalink\"></a>",
                EscapeAttribute(&attributes.id)
            )
            .unwrap();
        }
        s.push_str("</th-bb>");

        if has_children {
            s.push_str("</summary>");
            branches_to_html(s, treehouse, file_id, &branch.children);
            s.push_str("</details>");
        } else {
            s.push_str("</div>");
        }
    }
    s.push_str("</li>");
}

fn parse_attributes(treehouse: &mut Treehouse, file_id: usize, branch: &Branch) -> Attributes {
    let source = treehouse.get_source(file_id);

    let mut successfully_parsed = true;
    let mut attributes = if let Some(attributes) = &branch.attributes {
        toml_edit::de::from_str(&source[attributes.data.clone()]).unwrap_or_else(|error| {
            treehouse
                .diagnostics
                .push(toml_error_to_diagnostic(TomlError {
                    message: error.message().to_owned(),
                    span: error.span(),
                    file_id,
                    input_range: attributes.data.clone(),
                }));
            successfully_parsed = false;
            Attributes::default()
        })
    } else {
        Attributes::default()
    };
    let successfully_parsed = successfully_parsed;

    // Only check for attribute validity if the attributes were parsed successfully.
    if successfully_parsed {
        let attribute_warning_span = branch
            .attributes
            .as_ref()
            .map(|attributes| attributes.percent.clone())
            .unwrap_or(branch.kind_span.clone());

        // Check that every block has an ID.
        if attributes.id.is_empty() {
            attributes.id = format!("treehouse-missingno-{}", treehouse.next_missingno());
            treehouse.diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                code: Some("attr".into()),
                message: "branch does not have an `id` attribute".into(),
                labels: vec![Label {
                    style: LabelStyle::Primary,
                    file_id,
                    range: attribute_warning_span.clone(),
                    message: String::new(),
                }],
                notes: vec![
                    format!(
                        "note: a generated id `{}` will be used, but this id is unstable and will not persist across generations",
                        attributes.id
                    ),
                    format!("help: run `treehouse fix {}` to add missing ids to branches", treehouse.get_filename(file_id)),
                ],
            });
        }

        // Check that link-type blocks are `+`-type to facilitate lazy loading.
        if let Content::Link(_) = &attributes.content {
            if branch.kind == BranchKind::Expanded {
                treehouse.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    code: Some("attr".into()),
                    message: "`content.link` branch is expanded by default".into(),
                    labels: vec![Label {
                        style: LabelStyle::Primary,
                        file_id,
                        range: branch.kind_span.clone(),
                        message: String::new(),
                    }],
                    notes: vec![
                        "note: `content.link` branches should normally be collapsed to allow for lazy loading".into(),
                    ],
                });
            }
        }
    }
    attributes
}

pub fn branches_to_html(
    s: &mut String,
    treehouse: &mut Treehouse,
    file_id: FileId,
    branches: &[Branch],
) {
    s.push_str("<ul>");
    for child in branches {
        branch_to_html(s, treehouse, file_id, child);
    }
    s.push_str("</ul>");
}
