//! Djot -> HTML renderer adapted from the one in jotdown.
//! Made concrete to avoid generic hell, with added treehouse-specific features.

use std::fmt::Write;
use std::ops::Range;

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::diagnostic::LabelStyle;
use codespan_reporting::diagnostic::Severity;
use jotdown::Alignment;
use jotdown::Container;
use jotdown::Event;
use jotdown::LinkType;
use jotdown::ListKind;
use jotdown::OrderedListNumbering::*;
use jotdown::SpanLinkType;

use crate::config::Config;
use crate::config::ConfigDerivedData;
use crate::state::FileId;
use crate::state::Treehouse;

use super::highlight::highlight;

/// [`Render`] implementor that writes HTML output.
pub struct Renderer<'a> {
    pub config: &'a Config,
    pub config_derived_data: &'a mut ConfigDerivedData,
    pub treehouse: &'a mut Treehouse,
    pub file_id: FileId,
    pub page_id: String,
}

impl<'a> Renderer<'a> {
    pub fn render(self, events: &[(Event, Range<usize>)], out: &mut String) {
        let mut writer = Writer {
            renderer: self,
            raw: Raw::None,
            code_block: None,
            img_alt_text: 0,
            list_tightness: vec![],
            not_first_line: false,
            ignore_next_event: false,
        };

        for (event, range) in events {
            writer
                .render_event(event, range.clone(), out)
                .expect("formatting event into string should not fail");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Raw {
    #[default]
    None,
    Html,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CodeBlockKind {
    PlainText,
    SyntaxHighlight,
    LiterateProgram {
        program_name: String,
        placeholder_pic_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodeBlock<'a> {
    kind: CodeBlockKind,
    language: &'a str,
}

struct Writer<'a> {
    renderer: Renderer<'a>,

    raw: Raw,
    code_block: Option<CodeBlock<'a>>,
    img_alt_text: usize,
    list_tightness: Vec<bool>,
    not_first_line: bool,
    ignore_next_event: bool,
}

impl<'a> Writer<'a> {
    fn render_event(
        &mut self,
        e: &Event<'a>,
        range: Range<usize>,
        out: &mut String,
    ) -> std::fmt::Result {
        if let Event::Start(Container::Footnote { label: _ }, ..) = e {
            self.renderer.treehouse.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: Some("djot".into()),
                message: "Djot footnotes are not supported".into(),
                labels: vec![Label {
                    style: LabelStyle::Primary,
                    file_id: self.renderer.file_id,
                    range: range.clone(),
                    message: "".into(),
                }],
                notes: vec![],
            })
        }

        if matches!(&e, Event::Start(Container::LinkDefinition { .. }, ..)) {
            self.ignore_next_event = true;
            return Ok(());
        }

        if matches!(&e, Event::End(Container::LinkDefinition { .. })) {
            self.ignore_next_event = false;
            return Ok(());
        }

        // Completely omit section events. The treehouse's structure contains linkable ids in
        // branches instead.
        if matches!(
            &e,
            Event::Start(Container::Section { .. }, _) | Event::End(Container::Section { .. })
        ) {
            return Ok(());
        }

        if self.ignore_next_event {
            return Ok(());
        }

        match e {
            Event::Start(c, attrs) => {
                if c.is_block() && self.not_first_line {
                    out.push('\n');
                }
                if self.img_alt_text > 0 && !matches!(c, Container::Image(..)) {
                    return Ok(());
                }
                match &c {
                    Container::Blockquote => out.push_str("<blockquote"),
                    Container::List { kind, tight } => {
                        self.list_tightness.push(*tight);
                        match kind {
                            ListKind::Unordered | ListKind::Task => out.push_str("<ul"),
                            ListKind::Ordered {
                                numbering, start, ..
                            } => {
                                out.push_str("<ol");
                                if *start > 1 {
                                    write!(out, r#" start="{}""#, start)?;
                                }
                                if let Some(ty) = match numbering {
                                    Decimal => None,
                                    AlphaLower => Some('a'),
                                    AlphaUpper => Some('A'),
                                    RomanLower => Some('i'),
                                    RomanUpper => Some('I'),
                                } {
                                    write!(out, r#" type="{}""#, ty)?;
                                }
                            }
                        }
                    }
                    Container::ListItem | Container::TaskListItem { .. } => {
                        out.push_str("<li");
                    }
                    Container::DescriptionList => out.push_str("<dl"),
                    Container::DescriptionDetails => out.push_str("<dd"),
                    Container::Footnote { .. } => unreachable!(),
                    Container::Table => out.push_str("<table"),
                    Container::TableRow { .. } => out.push_str("<tr"),
                    Container::Section { .. } => {}
                    Container::Div { .. } => out.push_str("<div"),
                    Container::Paragraph => {
                        if matches!(self.list_tightness.last(), Some(true)) {
                            return Ok(());
                        }
                        out.push_str("<p");
                    }
                    Container::Heading { level, .. } => write!(out, "<h{}", level)?,
                    Container::TableCell { head: false, .. } => out.push_str("<td"),
                    Container::TableCell { head: true, .. } => out.push_str("<th"),
                    Container::Caption => out.push_str("<caption"),
                    Container::DescriptionTerm => out.push_str("<dt"),
                    Container::CodeBlock { language } => {
                        if let Some(program) = attrs.get(":program") {
                            self.code_block = Some(CodeBlock {
                                kind: CodeBlockKind::LiterateProgram {
                                    program_name: program.parts().collect(),
                                    placeholder_pic_id: attrs
                                        .get(":placeholder")
                                        .map(|value| value.parts().collect()),
                                },
                                language,
                            });
                            out.push_str("<th-literate-program");
                        } else {
                            self.code_block = Some(CodeBlock {
                                kind: match self.renderer.config.syntaxes.contains_key(*language) {
                                    true => CodeBlockKind::SyntaxHighlight,
                                    false => CodeBlockKind::PlainText,
                                },
                                language,
                            });
                            out.push_str("<pre");
                        }
                    }
                    Container::Span | Container::Math { .. } => out.push_str("<span"),
                    Container::Link(dst, ty) => {
                        if matches!(ty, LinkType::Span(SpanLinkType::Unresolved)) {
                            out.push_str("<a");
                            if let Some(resolved) = self.resolve_link(dst) {
                                out.push_str(r#" href=""#);
                                write_attr(&resolved, out);
                                out.push('"');
                            }
                        } else {
                            out.push_str(r#"<a href=""#);
                            if matches!(ty, LinkType::Email) {
                                out.push_str("mailto:");
                            }
                            write_attr(dst, out);
                            out.push('"');
                        }
                    }
                    Container::Image(..) => {
                        self.img_alt_text += 1;
                        if self.img_alt_text == 1 {
                            out.push_str(r#"<img class="pic""#);
                        } else {
                            return Ok(());
                        }
                    }
                    Container::Verbatim => out.push_str("<code"),
                    Container::RawBlock { format } | Container::RawInline { format } => {
                        self.raw = if format == &"html" {
                            Raw::Html
                        } else {
                            Raw::Other
                        };
                        return Ok(());
                    }
                    Container::Subscript => out.push_str("<sub"),
                    Container::Superscript => out.push_str("<sup"),
                    Container::Insert => out.push_str("<ins"),
                    Container::Delete => out.push_str("<del"),
                    Container::Strong => out.push_str("<strong"),
                    Container::Emphasis => out.push_str("<em"),
                    Container::Mark => out.push_str("<mark"),
                    Container::LinkDefinition { .. } => return Ok(()),
                }

                for (key, value) in attrs
                    .into_iter()
                    .filter(|(a, _)| !(*a == "class" || a.starts_with(':')))
                {
                    write!(out, r#" {}=""#, key)?;
                    value.parts().for_each(|part| write_attr(part, out));
                    out.push('"');
                }

                if attrs.into_iter().any(|(a, _)| a == "class")
                    || matches!(
                        c,
                        Container::Div { class } if !class.is_empty())
                    || matches!(c, |Container::Math { .. }| Container::List {
                        kind: ListKind::Task,
                        ..
                    } | Container::TaskListItem { .. })
                {
                    out.push_str(r#" class=""#);
                    let mut first_written = false;
                    if let Some(cls) = match c {
                        Container::List {
                            kind: ListKind::Task,
                            ..
                        } => Some("task-list"),
                        Container::TaskListItem { checked: false } => Some("unchecked"),
                        Container::TaskListItem { checked: true } => Some("checked"),
                        Container::Math { display: false } => Some("math inline"),
                        Container::Math { display: true } => Some("math display"),
                        _ => None,
                    } {
                        first_written = true;
                        out.push_str(cls);
                    }
                    for class in attrs
                        .into_iter()
                        .filter(|(a, _)| a == &"class")
                        .map(|(_, cls)| cls)
                    {
                        if first_written {
                            out.push(' ');
                        }
                        first_written = true;
                        class.parts().for_each(|part| write_attr(part, out));
                    }
                    // div class goes after classes from attrs
                    if let Container::Div { class } = c {
                        if !class.is_empty() {
                            if first_written {
                                out.push(' ');
                            }
                            out.push_str(class);
                        }
                    }
                    out.push('"');
                }

                match c {
                    Container::TableCell { alignment, .. }
                        if !matches!(alignment, Alignment::Unspecified) =>
                    {
                        let a = match alignment {
                            Alignment::Unspecified => unreachable!(),
                            Alignment::Left => "left",
                            Alignment::Center => "center",
                            Alignment::Right => "right",
                        };
                        write!(out, r#" style="text-align: {};">"#, a)?;
                    }
                    Container::CodeBlock { language } => {
                        if language.is_empty() {
                            out.push_str("><code>");
                        } else {
                            let code_block = self.code_block.as_ref().unwrap();
                            if let CodeBlockKind::LiterateProgram { program_name, .. } =
                                &code_block.kind
                            {
                                out.push_str(r#" data-program=""#);
                                write_attr(&self.renderer.page_id, out);
                                out.push(':');
                                write_attr(program_name, out);
                                out.push('"');

                                out.push_str(r#" data-language=""#);
                                write_attr(language, out);
                                out.push('"');

                                if *language == "output" {
                                    out.push_str(r#" data-mode="output""#);
                                } else {
                                    out.push_str(r#" data-mode="input""#);
                                }
                            }

                            out.push('>');

                            if let CodeBlockKind::LiterateProgram {
                                placeholder_pic_id: Some(placeholder_pic_id),
                                ..
                            } = &code_block.kind
                            {
                                out.push_str(
                                    r#"<img class="placeholder-image" loading="lazy" src=""#,
                                );

                                let filename = self.renderer.config.pics.get(placeholder_pic_id);
                                let pic_url = filename
                                    .and_then(|filename| {
                                        self.renderer
                                            .config_derived_data
                                            .static_urls
                                            .get(&format!("pic/{filename}"))
                                            .ok()
                                    })
                                    .unwrap_or_default();
                                write_attr(&pic_url, out);
                                out.push('"');

                                let image_size = filename.and_then(|filename| {
                                    self.renderer
                                        .config_derived_data
                                        .image_size(&format!("static/pic/{filename}"))
                                });
                                if let Some(image_size) = image_size {
                                    write!(
                                        out,
                                        r#" width="{}" height="{}""#,
                                        image_size.width, image_size.height
                                    )?;
                                }

                                out.push('>');
                            }

                            if let (CodeBlockKind::LiterateProgram { .. }, "output") =
                                (&code_block.kind, *language)
                            {
                                out.push_str(r#"<pre class="placeholder-console">"#);
                            } else {
                                out.push_str(r#"<code class="language-"#);
                                write_attr(language, out);
                                if self.renderer.config.syntaxes.contains_key(*language) {
                                    out.push_str(" th-syntax-highlighting");
                                }
                                out.push_str(r#"">"#);
                            }
                        }
                    }
                    Container::Image(..) => {
                        if self.img_alt_text == 1 {
                            out.push_str(r#" alt=""#);
                        }
                    }
                    Container::Math { display } => {
                        out.push_str(if *display { r#">\["# } else { r#">\("# });
                    }
                    _ => out.push('>'),
                }
            }
            Event::End(c) => {
                if c.is_block_container() {
                    out.push('\n');
                }
                if self.img_alt_text > 0 && !matches!(c, Container::Image(..)) {
                    return Ok(());
                }
                match c {
                    Container::Blockquote => out.push_str("</blockquote>"),
                    Container::List { kind, .. } => {
                        self.list_tightness.pop();
                        match kind {
                            ListKind::Unordered | ListKind::Task => out.push_str("</ul>"),
                            ListKind::Ordered { .. } => out.push_str("</ol>"),
                        }
                    }
                    Container::ListItem | Container::TaskListItem { .. } => {
                        out.push_str("</li>");
                    }
                    Container::DescriptionList => out.push_str("</dl>"),
                    Container::DescriptionDetails => out.push_str("</dd>"),
                    Container::Footnote { .. } => unreachable!(),
                    Container::Table => out.push_str("</table>"),
                    Container::TableRow { .. } => out.push_str("</tr>"),
                    Container::Section { .. } => {}
                    Container::Div { .. } => out.push_str("</div>"),
                    Container::Paragraph => {
                        if matches!(self.list_tightness.last(), Some(true)) {
                            return Ok(());
                        }
                        out.push_str("</p>");
                    }
                    Container::Heading { level, .. } => write!(out, "</h{}>", level)?,
                    Container::TableCell { head: false, .. } => out.push_str("</td>"),
                    Container::TableCell { head: true, .. } => out.push_str("</th>"),
                    Container::Caption => out.push_str("</caption>"),
                    Container::DescriptionTerm => out.push_str("</dt>"),
                    Container::CodeBlock { language } => {
                        let code_block = self.code_block.take().unwrap();

                        out.push_str(match &code_block.kind {
                            CodeBlockKind::PlainText | CodeBlockKind::SyntaxHighlight => {
                                "</code></pre>"
                            }
                            CodeBlockKind::LiterateProgram { .. } if *language == "output" => {
                                "</pre></th-literate-program>"
                            }
                            CodeBlockKind::LiterateProgram { .. } => {
                                "</code></th-literate-program>"
                            }
                        });
                    }
                    Container::Span => out.push_str("</span>"),
                    Container::Link(..) => out.push_str("</a>"),
                    Container::Image(src, link_type) => {
                        if self.img_alt_text == 1 {
                            if !src.is_empty() {
                                out.push_str(r#"" src=""#);
                                if let SpanLinkType::Unresolved = link_type {
                                    if let Some(resolved) = self.resolve_link(src) {
                                        write_attr(&resolved, out);
                                    } else {
                                        write_attr(src, out);
                                    }
                                } else {
                                    write_attr(src, out);
                                }
                            }
                            out.push_str(r#"">"#);
                        }
                        self.img_alt_text -= 1;
                    }
                    Container::Verbatim => out.push_str("</code>"),
                    Container::Math { display } => {
                        out.push_str(if *display {
                            r#"\]</span>"#
                        } else {
                            r#"\)</span>"#
                        });
                    }
                    Container::RawBlock { .. } | Container::RawInline { .. } => {
                        self.raw = Raw::None;
                    }
                    Container::Subscript => out.push_str("</sub>"),
                    Container::Superscript => out.push_str("</sup>"),
                    Container::Insert => out.push_str("</ins>"),
                    Container::Delete => out.push_str("</del>"),
                    Container::Strong => out.push_str("</strong>"),
                    Container::Emphasis => out.push_str("</em>"),
                    Container::Mark => out.push_str("</mark>"),
                    Container::LinkDefinition { .. } => unreachable!(),
                }
            }
            Event::Str(s) => match self.raw {
                Raw::None if self.img_alt_text > 0 => write_attr(s, out),
                Raw::None => {
                    let syntax = self.code_block.as_ref().and_then(|code_block| {
                        self.renderer.config.syntaxes.get(code_block.language)
                    });
                    if let Some(syntax) = syntax {
                        // TODO djot: make highlight infallible
                        highlight(out, syntax, s).map_err(|_| std::fmt::Error)?;
                    } else {
                        write_text(s, out);
                    }
                }
                Raw::Html => out.push_str(s),
                Raw::Other => {}
            },
            Event::FootnoteReference(_label) => {
                self.renderer.treehouse.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: Some("djot".into()),
                    message: "Djot footnotes are unsupported".into(),
                    labels: vec![Label {
                        style: LabelStyle::Primary,
                        file_id: self.renderer.file_id,
                        range,
                        message: "".into(),
                    }],
                    notes: vec![],
                });
            }
            Event::Symbol(sym) => {
                if let Some(filename) = self.renderer.config.emoji.get(sym.as_ref()) {
                    let branch_id = self
                        .renderer
                        .treehouse
                        .branches_by_named_id
                        .get(&format!("emoji/{sym}"))
                        .copied();

                    if let Some(branch) =
                        branch_id.map(|id| self.renderer.treehouse.tree.branch(id))
                    {
                        out.push_str(r#"<a href=""#);
                        write_attr(&self.renderer.config.site, out);
                        out.push_str("/b?");
                        write_attr(&branch.attributes.id, out);
                        out.push_str(r#"">"#)
                    }

                    let url = self
                        .renderer
                        .config_derived_data
                        .static_urls
                        .get(&format!("emoji/{filename}"))
                        .unwrap_or_default();

                    // TODO: this could do with better alt text
                    write!(
                        out,
                        r#"<img data-cast="emoji" title=":{sym}:" alt="{sym}" src=""#
                    )?;
                    write_attr(&url, out);
                    out.push('"');

                    if let Some(image_size) = self
                        .renderer
                        .config_derived_data
                        .image_size(&format!("static/emoji/{filename}"))
                    {
                        write!(
                            out,
                            r#" width="{}" height="{}""#,
                            image_size.width, image_size.height
                        )?;
                    }

                    out.push('>');

                    if branch_id.is_some() {
                        out.push_str("</a>");
                    }
                } else {
                    write!(
                        out,
                        r#"<span class="th-emoji-unknown" title="this emoji does not exist… yet!">:{sym}:</span>"#,
                    )?
                }
            }
            Event::LeftSingleQuote => out.push('‘'),
            Event::RightSingleQuote => out.push('’'),
            Event::LeftDoubleQuote => out.push('“'),
            Event::RightDoubleQuote => out.push('”'),
            Event::Ellipsis => out.push('…'),
            Event::EnDash => out.push('–'),
            Event::EmDash => out.push('—'),
            Event::NonBreakingSpace => out.push_str("&nbsp;"),
            Event::Hardbreak => out.push_str("<br>\n"),
            Event::Softbreak => out.push('\n'),
            Event::Escape | Event::Blankline => {}
            Event::ThematicBreak(attrs) => {
                if self.not_first_line {
                    out.push('\n');
                }
                out.push_str("<hr");
                for (a, v) in attrs {
                    write!(out, r#" {}=""#, a)?;
                    v.parts().for_each(|part| write_attr(part, out));
                    out.push('"');
                }
                out.push('>');
            }
        }
        self.not_first_line = true;

        Ok(())
    }

    fn resolve_link(&self, link: &str) -> Option<String> {
        let Renderer {
            config,
            config_derived_data,
            treehouse,
            ..
        } = &self.renderer;
        link.split_once(':').and_then(|(kind, linked)| match kind {
            "def" => config.defs.get(linked).cloned(),
            "branch" => treehouse
                .branches_by_named_id
                .get(linked)
                .map(|&branch_id| {
                    format!(
                        "{}/b?{}",
                        config.site,
                        treehouse.tree.branch(branch_id).attributes.id
                    )
                }),
            "page" => Some(config.page_url(linked)),
            "pic" => config.pics.get(linked).and_then(|filename| {
                config_derived_data
                    .static_urls
                    .get(&format!("pic/{filename}"))
                    .ok()
            }),
            _ => None,
        })
    }
}

fn write_text(s: &str, out: &mut String) {
    write_escape(s, false, out)
}

fn write_attr(s: &str, out: &mut String) {
    write_escape(s, true, out)
}

fn write_escape(mut s: &str, escape_quotes: bool, out: &mut String) {
    let mut ent = "";
    while let Some(i) = s.find(|c| {
        match c {
            '<' => Some("&lt;"),
            '>' => Some("&gt;"),
            '&' => Some("&amp;"),
            '"' if escape_quotes => Some("&quot;"),
            _ => None,
        }
        .map_or(false, |s| {
            ent = s;
            true
        })
    }) {
        out.push_str(&s[..i]);
        out.push_str(ent);
        s = &s[i + 1..];
    }
    out.push_str(s);
}
