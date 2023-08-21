use std::{ops::Range, str::FromStr};

use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle, Severity};
use treehouse_format::ast::Roots;

use crate::state::{toml_error_to_diagnostic, FileId, TomlError, Treehouse};

pub struct ErrorsEmitted;

pub fn parse_tree_with_diagnostics(
    treehouse: &mut Treehouse,
    file_id: FileId,
) -> Result<Roots, ErrorsEmitted> {
    let input = treehouse.source(file_id);
    Roots::parse(&mut treehouse_format::pull::Parser { input, position: 0 }).map_err(|error| {
        treehouse.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: Some("tree".into()),
            message: error.kind.to_string(),
            labels: vec![Label {
                style: LabelStyle::Primary,
                file_id,
                range: error.range,
                message: String::new(),
            }],
            notes: vec![],
        });
        ErrorsEmitted
    })
}

pub fn parse_toml_with_diagnostics(
    treehouse: &mut Treehouse,
    file_id: FileId,
    range: Range<usize>,
) -> Result<toml_edit::Document, ErrorsEmitted> {
    let input = &treehouse.source(file_id)[range.clone()];
    toml_edit::Document::from_str(input).map_err(|error| {
        treehouse
            .diagnostics
            .push(toml_error_to_diagnostic(TomlError {
                message: error.message().to_owned(),
                span: error.span(),
                file_id,
                input_range: range.clone(),
            }));
        ErrorsEmitted
    })
}
