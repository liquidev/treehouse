// NOTE: This code is pasted pretty much verbatim from pulldown-cmark but tweaked to have my own
// cool additions.

// Copyright 2015 Google Inc. All rights reserved.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! HTML renderer that takes an iterator of events as input.

use std::collections::HashMap;
use std::io;

use pulldown_cmark::escape::{escape_href, escape_html, StrWrite};
use pulldown_cmark::{Alignment, CodeBlockKind, Event, LinkType, Tag};
use pulldown_cmark::{CowStr, Event::*};

use crate::config::{Config, ConfigDerivedData, ImageSize};
use crate::html::highlight::highlight;
use crate::state::Treehouse;

enum TableState {
    Head,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CodeBlockState<'a> {
    NotInCodeBlock,
    InCodeBlock(Option<CowStr<'a>>),
}

struct HtmlWriter<'a, I, W> {
    treehouse: &'a Treehouse,
    config: &'a Config,
    config_derived_data: &'a mut ConfigDerivedData,
    page_id: &'a str,

    /// Iterator supplying events.
    iter: I,

    /// Writer to write to.
    writer: W,

    /// Whether or not the last write wrote a newline.
    end_newline: bool,

    table_state: TableState,
    table_alignments: Vec<Alignment>,
    table_cell_index: usize,
    numbers: HashMap<CowStr<'a>, usize>,

    code_block_state: CodeBlockState<'a>,
}

impl<'a, I, W> HtmlWriter<'a, I, W>
where
    I: Iterator<Item = Event<'a>>,
    W: StrWrite,
{
    fn new(
        treehouse: &'a Treehouse,
        config: &'a Config,
        config_derived_data: &'a mut ConfigDerivedData,
        page_id: &'a str,
        iter: I,
        writer: W,
    ) -> Self {
        Self {
            treehouse,
            config,
            config_derived_data,
            page_id,

            iter,
            writer,
            end_newline: true,
            table_state: TableState::Head,
            table_alignments: vec![],
            table_cell_index: 0,
            numbers: HashMap::new(),
            code_block_state: CodeBlockState::NotInCodeBlock,
        }
    }

    /// Writes a new line.
    fn write_newline(&mut self) -> io::Result<()> {
        self.end_newline = true;
        self.writer.write_str("\n")
    }

    /// Writes a buffer, and tracks whether or not a newline was written.
    #[inline]
    fn write(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_str(s)?;

        if !s.is_empty() {
            self.end_newline = s.ends_with('\n');
        }
        Ok(())
    }

    fn run(mut self) -> io::Result<()> {
        while let Some(event) = self.iter.next() {
            match event {
                Start(tag) => {
                    self.start_tag(tag)?;
                }
                End(tag) => {
                    self.end_tag(tag)?;
                }
                Text(text) => {
                    self.run_text(&text)?;
                    self.end_newline = text.ends_with('\n');
                }
                Code(text) => {
                    self.write("<code>")?;
                    escape_html(&mut self.writer, &text)?;
                    self.write("</code>")?;
                }
                Html(html) => {
                    self.write(&html)?;
                }
                SoftBreak => {
                    self.write_newline()?;
                }
                HardBreak => {
                    self.write("<br />\n")?;
                }
                Rule => {
                    if self.end_newline {
                        self.write("<hr />\n")?;
                    } else {
                        self.write("\n<hr />\n")?;
                    }
                }
                FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    self.write("<sup class=\"footnote-reference\"><a href=\"#")?;
                    escape_html(&mut self.writer, &name)?;
                    self.write("\">")?;
                    let number = *self.numbers.entry(name).or_insert(len);
                    write!(&mut self.writer, "{}", number)?;
                    self.write("</a></sup>")?;
                }
                TaskListMarker(true) => {
                    self.write("<input disabled=\"\" type=\"checkbox\" checked=\"\"/>\n")?;
                }
                TaskListMarker(false) => {
                    self.write("<input disabled=\"\" type=\"checkbox\"/>\n")?;
                }
            }
        }
        Ok(())
    }

    /// Writes the start of an HTML tag.
    fn start_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {
                if self.end_newline {
                    self.write("<p>")
                } else {
                    self.write("\n<p>")
                }
            }
            Tag::Heading(level, id, classes) => {
                if self.end_newline {
                    self.end_newline = false;
                    self.write("<")?;
                } else {
                    self.write("\n<")?;
                }
                write!(&mut self.writer, "{}", level)?;
                if let Some(id) = id {
                    self.write(" id=\"")?;
                    escape_html(&mut self.writer, id)?;
                    self.write("\"")?;
                }
                let mut classes = classes.iter();
                if let Some(class) = classes.next() {
                    self.write(" class=\"")?;
                    escape_html(&mut self.writer, class)?;
                    for class in classes {
                        self.write(" ")?;
                        escape_html(&mut self.writer, class)?;
                    }
                    self.write("\"")?;
                }
                self.write(">")
            }
            Tag::Table(alignments) => {
                self.table_alignments = alignments;
                self.write("<table>")
            }
            Tag::TableHead => {
                self.table_state = TableState::Head;
                self.table_cell_index = 0;
                self.write("<thead><tr>")
            }
            Tag::TableRow => {
                self.table_cell_index = 0;
                self.write("<tr>")
            }
            Tag::TableCell => {
                match self.table_state {
                    TableState::Head => {
                        self.write("<th")?;
                    }
                    TableState::Body => {
                        self.write("<td")?;
                    }
                }
                match self.table_alignments.get(self.table_cell_index) {
                    Some(&Alignment::Left) => self.write(" style=\"text-align: left\">"),
                    Some(&Alignment::Center) => self.write(" style=\"text-align: center\">"),
                    Some(&Alignment::Right) => self.write(" style=\"text-align: right\">"),
                    _ => self.write(">"),
                }
            }
            Tag::BlockQuote => {
                if self.end_newline {
                    self.write("<blockquote>\n")
                } else {
                    self.write("\n<blockquote>\n")
                }
            }
            Tag::CodeBlock(info) => {
                self.code_block_state = CodeBlockState::InCodeBlock(None);
                if !self.end_newline {
                    self.write_newline()?;
                }
                match info {
                    CodeBlockKind::Fenced(language) => {
                        self.code_block_state = CodeBlockState::InCodeBlock(Some(language.clone()));
                        match CodeBlockMode::parse(&language) {
                            CodeBlockMode::PlainText => self.write("<pre><code>"),
                            CodeBlockMode::SyntaxHighlightOnly { language } => {
                                self.write("<pre><code class=\"language-")?;
                                escape_html(&mut self.writer, language)?;
                                if self.config.syntaxes.contains_key(language) {
                                    self.write(" th-syntax-highlighting")?;
                                }
                                self.write("\">")
                            }
                            CodeBlockMode::LiterateProgram {
                                language,
                                kind,
                                program_name,
                            } => {
                                self.write(match &kind {
                                    LiterateCodeKind::Input => {
                                        "<th-literate-program data-mode=\"input\" "
                                    }
                                    LiterateCodeKind::Output { .. } => {
                                        "<th-literate-program data-mode=\"output\" "
                                    }
                                })?;
                                self.write("data-program=\"")?;
                                escape_href(&mut self.writer, self.page_id)?;
                                self.write(":")?;
                                escape_html(&mut self.writer, program_name)?;
                                self.write("\" data-language=\"")?;
                                escape_html(&mut self.writer, language)?;
                                self.write("\" role=\"code\">")?;

                                if let LiterateCodeKind::Output { placeholder_pic_id } = kind {
                                    if !placeholder_pic_id.is_empty() {
                                        self.write("<img class=\"placeholder-image\" loading=\"lazy\" src=\"")?;
                                        escape_html(
                                            &mut self.writer,
                                            &self.config.pic_url(placeholder_pic_id),
                                        )?;
                                        self.write("\"")?;
                                        if let Some(ImageSize { width, height }) = self
                                            .config_derived_data
                                            .pic_size(self.config, placeholder_pic_id)
                                        {
                                            self.write(&format!(
                                                " width=\"{width}\" height=\"{height}\""
                                            ))?;
                                        }
                                        self.write(">")?;
                                    }
                                }

                                self.write("<pre class=\"placeholder-console\">")?;
                                Ok(())
                            }
                        }
                    }
                    CodeBlockKind::Indented => self.write("<pre><code>"),
                }
            }
            Tag::List(Some(1)) => {
                if self.end_newline {
                    self.write("<ol>\n")
                } else {
                    self.write("\n<ol>\n")
                }
            }
            Tag::List(Some(start)) => {
                if self.end_newline {
                    self.write("<ol start=\"")?;
                } else {
                    self.write("\n<ol start=\"")?;
                }
                write!(&mut self.writer, "{}", start)?;
                self.write("\">\n")
            }
            Tag::List(None) => {
                if self.end_newline {
                    self.write("<ul>\n")
                } else {
                    self.write("\n<ul>\n")
                }
            }
            Tag::Item => {
                if self.end_newline {
                    self.write("<li>")
                } else {
                    self.write("\n<li>")
                }
            }
            Tag::Emphasis => self.write("<em>"),
            Tag::Strong => self.write("<strong>"),
            Tag::Strikethrough => self.write("<del>"),
            Tag::Link(LinkType::Email, dest, title) => {
                self.write("<a href=\"mailto:")?;
                escape_href(&mut self.writer, &dest)?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    escape_html(&mut self.writer, &title)?;
                }
                self.write("\">")
            }
            Tag::Link(_link_type, dest, title) => {
                self.write("<a href=\"")?;
                escape_href(&mut self.writer, &dest)?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    escape_html(&mut self.writer, &title)?;
                }
                self.write("\">")
            }
            Tag::Image(_link_type, dest, title) => {
                self.write("<img class=\"pic\" src=\"")?;
                escape_href(&mut self.writer, &dest)?;
                self.write("\" alt=\"")?;
                self.raw_text()?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    escape_html(&mut self.writer, &title)?;
                }
                self.write("\" />")
            }
            Tag::FootnoteDefinition(name) => {
                if self.end_newline {
                    self.write("<div class=\"footnote-definition\" id=\"")?;
                } else {
                    self.write("\n<div class=\"footnote-definition\" id=\"")?;
                }
                escape_html(&mut self.writer, &name)?;
                self.write("\"><sup class=\"footnote-definition-label\">")?;
                let len = self.numbers.len() + 1;
                let number = *self.numbers.entry(name).or_insert(len);
                write!(&mut self.writer, "{}", number)?;
                self.write("</sup>")
            }
        }
    }

    fn end_tag(&mut self, tag: Tag) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {
                self.write("</p>\n")?;
            }
            Tag::Heading(level, _id, _classes) => {
                self.write("</")?;
                write!(&mut self.writer, "{}", level)?;
                self.write(">\n")?;
            }
            Tag::Table(_) => {
                self.write("</tbody></table>\n")?;
            }
            Tag::TableHead => {
                self.write("</tr></thead><tbody>\n")?;
                self.table_state = TableState::Body;
            }
            Tag::TableRow => {
                self.write("</tr>\n")?;
            }
            Tag::TableCell => {
                match self.table_state {
                    TableState::Head => {
                        self.write("</th>")?;
                    }
                    TableState::Body => {
                        self.write("</td>")?;
                    }
                }
                self.table_cell_index += 1;
            }
            Tag::BlockQuote => {
                self.write("</blockquote>\n")?;
            }
            Tag::CodeBlock(kind) => {
                self.write(match kind {
                    CodeBlockKind::Fenced(language) => match CodeBlockMode::parse(&language) {
                        CodeBlockMode::LiterateProgram { .. } => "</pre></th-literate-program>",
                        _ => "</code></pre>",
                    },
                    _ => "</code></pre>\n",
                })?;
                self.code_block_state = CodeBlockState::NotInCodeBlock;
            }
            Tag::List(Some(_)) => {
                self.write("</ol>\n")?;
            }
            Tag::List(None) => {
                self.write("</ul>\n")?;
            }
            Tag::Item => {
                self.write("</li>\n")?;
            }
            Tag::Emphasis => {
                self.write("</em>")?;
            }
            Tag::Strong => {
                self.write("</strong>")?;
            }
            Tag::Strikethrough => {
                self.write("</del>")?;
            }
            Tag::Link(_, _, _) => {
                self.write("</a>")?;
            }
            Tag::Image(_, _, _) => (), // shouldn't happen, handled in start
            Tag::FootnoteDefinition(_) => {
                self.write("</div>\n")?;
            }
        }
        Ok(())
    }

    fn run_text(&mut self, text: &str) -> io::Result<()> {
        struct EmojiParser<'a> {
            text: &'a str,
            position: usize,
        }

        enum Token<'a> {
            Text(&'a str),
            Emoji(&'a str),
        }

        impl<'a> EmojiParser<'a> {
            fn current(&self) -> Option<char> {
                self.text[self.position..].chars().next()
            }

            fn next_token(&mut self) -> Option<Token<'a>> {
                match self.current() {
                    Some(':') => {
                        let text_start = self.position;
                        self.position += 1;
                        if self.current().is_some_and(|c| c.is_alphabetic()) {
                            let name_start = self.position;
                            while let Some(c) = self.current() {
                                if c.is_alphanumeric() || c == '_' {
                                    self.position += c.len_utf8();
                                } else {
                                    break;
                                }
                            }
                            if self.current() == Some(':') {
                                let name_end = self.position;
                                self.position += 1;
                                Some(Token::Emoji(&self.text[name_start..name_end]))
                            } else {
                                Some(Token::Text(&self.text[text_start..self.position]))
                            }
                        } else {
                            Some(Token::Text(&self.text[text_start..self.position]))
                        }
                    }
                    Some(_) => {
                        let start = self.position;
                        while let Some(c) = self.current() {
                            if c == ':' {
                                break;
                            } else {
                                self.position += c.len_utf8();
                            }
                        }
                        let end = self.position;
                        Some(Token::Text(&self.text[start..end]))
                    }
                    None => None,
                }
            }
        }

        if let CodeBlockState::InCodeBlock(language) = &self.code_block_state {
            let code_block_mode = language
                .as_ref()
                .map(|language| CodeBlockMode::parse(language));
            let highlighting_language = code_block_mode
                .as_ref()
                .and_then(|mode| mode.highlighting_language());
            let syntax =
                highlighting_language.and_then(|language| self.config.syntaxes.get(language));
            if let Some(syntax) = syntax {
                highlight(&mut self.writer, syntax, text)?;
            } else {
                escape_html(&mut self.writer, text)?;
            }
        } else {
            let mut parser = EmojiParser { text, position: 0 };
            while let Some(token) = parser.next_token() {
                match token {
                    Token::Text(text) => escape_html(&mut self.writer, text)?,
                    Token::Emoji(name) => {
                        if let Some(filename) = self.config.emoji.get(name) {
                            let branch_id = self
                                .treehouse
                                .branches_by_named_id
                                .get(&format!("emoji/{name}"))
                                .copied();
                            if let Some(branch) = branch_id.map(|id| self.treehouse.tree.branch(id))
                            {
                                self.writer.write_str("<a href=\"")?;
                                escape_html(&mut self.writer, &self.config.site)?;
                                self.writer.write_str("/b?")?;
                                escape_html(&mut self.writer, &branch.attributes.id)?;
                                self.writer.write_str("\">")?;
                            }

                            self.writer
                                .write_str("<img data-cast=\"emoji\" title=\":")?;
                            escape_html(&mut self.writer, name)?;
                            self.writer.write_str(":\" src=\"")?;
                            let url = self
                                .config_derived_data
                                .static_urls
                                .get(&format!("emoji/{filename}"))
                                .unwrap_or_default();
                            escape_html(&mut self.writer, &url)?;
                            self.writer.write_str("\" alt=\"")?;
                            escape_html(&mut self.writer, name)?;
                            if let Some(image_size) = self
                                .config_derived_data
                                .image_size(&format!("static/emoji/{filename}"))
                            {
                                write!(
                                    self.writer,
                                    "\" width=\"{}\" height=\"{}",
                                    image_size.width, image_size.height
                                )?;
                            }
                            self.writer.write_str("\">")?;

                            if branch_id.is_some() {
                                self.writer.write_str("</a>")?;
                            }
                        } else {
                            self.writer.write_str(":")?;
                            escape_html(&mut self.writer, name)?;
                            self.writer.write_str(":")?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // run raw text, consuming end tag
    fn raw_text(&mut self) -> io::Result<()> {
        let mut nest = 0;
        while let Some(event) = self.iter.next() {
            match event {
                Start(_) => nest += 1,
                End(_) => {
                    if nest == 0 {
                        break;
                    }
                    nest -= 1;
                }
                Html(text) | Code(text) | Text(text) => {
                    escape_html(&mut self.writer, &text)?;
                    self.end_newline = text.ends_with('\n');
                }
                SoftBreak | HardBreak | Rule => {
                    self.write(" ")?;
                }
                FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    let number = *self.numbers.entry(name).or_insert(len);
                    write!(&mut self.writer, "[{}]", number)?;
                }
                TaskListMarker(true) => self.write("[x]")?,
                TaskListMarker(false) => self.write("[ ]")?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiterateCodeKind<'a> {
    Input,
    Output { placeholder_pic_id: &'a str },
}

enum CodeBlockMode<'a> {
    PlainText,
    SyntaxHighlightOnly {
        language: &'a str,
    },
    LiterateProgram {
        language: &'a str,
        kind: LiterateCodeKind<'a>,
        program_name: &'a str,
    },
}

impl<'a> CodeBlockMode<'a> {
    fn parse(language: &'a str) -> CodeBlockMode<'a> {
        if language.is_empty() {
            CodeBlockMode::PlainText
        } else if let Some((language, program_name)) = language.split_once(' ') {
            let (program_name, placeholder_pic_id) =
                program_name.split_once(' ').unwrap_or((program_name, ""));
            CodeBlockMode::LiterateProgram {
                language,
                kind: if language == "output" {
                    LiterateCodeKind::Output { placeholder_pic_id }
                } else {
                    LiterateCodeKind::Input
                },
                program_name: program_name.split(' ').next().unwrap(),
            }
        } else {
            CodeBlockMode::SyntaxHighlightOnly { language }
        }
    }

    fn highlighting_language(&self) -> Option<&str> {
        if let CodeBlockMode::LiterateProgram { language, .. }
        | CodeBlockMode::SyntaxHighlightOnly { language } = self
        {
            Some(language)
        } else {
            None
        }
    }
}

/// Iterate over an `Iterator` of `Event`s, generate HTML for each `Event`, and
/// push it to a `String`.
///
/// # Examples
///
/// ```
/// use pulldown_cmark::{html, Parser};
///
/// let markdown_str = r#"
/// hello
/// =====
///
/// * alpha
/// * beta
/// "#;
/// let parser = Parser::new(markdown_str);
///
/// let mut html_buf = String::new();
/// html::push_html(&mut html_buf, parser);
///
/// assert_eq!(html_buf, r#"<h1>hello</h1>
/// <ul>
/// <li>alpha</li>
/// <li>beta</li>
/// </ul>
/// "#);
/// ```
pub fn push_html<'a, I>(
    s: &mut String,
    treehouse: &'a Treehouse,
    config: &'a Config,
    config_derived_data: &'a mut ConfigDerivedData,
    page_id: &'a str,
    iter: I,
) where
    I: Iterator<Item = Event<'a>>,
{
    HtmlWriter::new(treehouse, config, config_derived_data, page_id, iter, s)
        .run()
        .unwrap();
}
