use std::{borrow::Cow, fmt::Write};

use jotdown::Render;
use pulldown_cmark::{BrokenLink, LinkType};
use treehouse_format::pull::BranchKind;

use crate::{
    cli::Paths,
    config::{Config, ConfigDerivedData, Markup},
    html::EscapeAttribute,
    state::{FileId, Treehouse},
    tree::{
        attributes::{Content, Stage},
        mini_template, SemaBranchId,
    },
};

use super::{djot, markdown, EscapeHtml};

pub fn branch_to_html(
    s: &mut String,
    treehouse: &mut Treehouse,
    config: &Config,
    config_derived_data: &mut ConfigDerivedData,
    paths: &Paths<'_>,
    file_id: FileId,
    branch_id: SemaBranchId,
) {
    let source = treehouse.source(file_id);
    let branch = treehouse.tree.branch(branch_id);

    if !cfg!(debug_assertions) && branch.attributes.stage == Stage::Draft {
        return;
    }

    let has_children =
        !branch.children.is_empty() || matches!(branch.attributes.content, Content::Link(_));

    let class = if has_children { "branch" } else { "leaf" };
    let mut class = String::from(class);
    if !branch.attributes.classes.branch.is_empty() {
        class.push(' ');
        class.push_str(&branch.attributes.classes.branch);
    }

    if branch.attributes.stage == Stage::Draft {
        class.push_str(" draft");
    }

    let component = if let Content::Link(_) = branch.attributes.content {
        "b-linked"
    } else {
        "b"
    };
    let component = if !branch.attributes.cast.is_empty() {
        Cow::Owned(format!("{component} {}", branch.attributes.cast))
    } else {
        Cow::Borrowed(component)
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

    let mut data_attributes = String::new();
    for (key, value) in &branch.attributes.data {
        write!(
            data_attributes,
            " data-{key}=\"{}\"",
            EscapeAttribute(value)
        )
        .unwrap();
    }

    write!(
        s,
        "<li data-cast=\"{component}\" class=\"{class}\" id=\"{}\"{linked_branch}{do_not_persist}{data_attributes}>",
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
        let mut final_markup = String::with_capacity(raw_block_content.len());
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

            final_markup.push_str(&line[space_count..]);
            final_markup.push('\n');
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
                                        "{}/b?{}",
                                        config.site,
                                        treehouse.tree.branch(branch_id).attributes.id
                                    )
                                    .into(),
                                    "".into(),
                                )
                            }),
                        "page" => Some((config.page_url(linked).into(), "".into())),
                        "pic" => config.pics.get(linked).map(|filename| {
                            (
                                // NOTE: We can't generate a URL with a hash here yet, because we
                                // cannot access ConfigDerivedData here due to it being borrowed
                                // by the Markdown parser.
                                format!("{}/static/pic/{}", config.site, &filename).into(),
                                "".into(),
                            )
                        }),
                        _ => None,
                    })
            } else {
                None
            }
        };
        if branch.attributes.template {
            final_markup = mini_template::render(config, treehouse, paths, &final_markup);
        }
        s.push_str("<th-bc>");
        match config.markup {
            Markup::Markdown => {
                let markdown_parser = pulldown_cmark::Parser::new_with_broken_link_callback(
                    &final_markup,
                    {
                        use pulldown_cmark::Options;
                        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES
                    },
                    Some(broken_link_callback),
                );
                markdown::push_html(
                    s,
                    treehouse,
                    config,
                    config_derived_data,
                    treehouse.tree_path(file_id).expect(".tree file expected"),
                    markdown_parser,
                )
            }
            Markup::Djot => {
                let events: Vec<_> = jotdown::Parser::new(&final_markup)
                    .into_offset_iter()
                    .collect();
                djot::Renderer {
                    page_id: treehouse
                        .tree_path(file_id)
                        .expect(".tree file expected")
                        .to_owned(),

                    config,
                    config_derived_data,
                    treehouse,
                    file_id,
                }
                .render(&events, s);
            }
        };

        let branch = treehouse.tree.branch(branch_id);
        if let Content::Link(link) = &branch.attributes.content {
            write!(
                s,
                "<noscript><a class=\"navigate icon-go\" href=\"{}/{}\">Go to linked tree: <code>{}</code></a></noscript>",
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
                    "<a class=\"icon icon-go\" href=\"{}/{}\" title=\"linked tree\"></a>",
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
                    branch_to_html(
                        s,
                        treehouse,
                        config,
                        config_derived_data,
                        paths,
                        file_id,
                        child_id,
                    );
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
    config_derived_data: &mut ConfigDerivedData,
    paths: &Paths<'_>,
    file_id: FileId,
    branches: &[SemaBranchId],
) {
    s.push_str("<ul>");
    for &child in branches {
        branch_to_html(
            s,
            treehouse,
            config,
            config_derived_data,
            paths,
            file_id,
            child,
        );
    }
    s.push_str("</ul>");
}
