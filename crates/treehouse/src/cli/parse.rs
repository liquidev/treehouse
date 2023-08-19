use std::{ops::Range, str::FromStr};

use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle, Severity};
use treehouse_format::ast::Roots;

use super::diagnostics::{Diagnosis, FileId};

pub struct ErrorsEmitted;

pub fn parse_tree_with_diagnostics(
    diagnosis: &mut Diagnosis,
    file_id: FileId,
) -> Result<Roots, ErrorsEmitted> {
    let input = diagnosis.get_source(file_id);
    Roots::parse(&mut treehouse_format::pull::Parser { input, position: 0 }).map_err(|error| {
        diagnosis.diagnostics.push(Diagnostic {
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
    diagnosis: &mut Diagnosis,
    file_id: FileId,
    range: Range<usize>,
) -> Result<toml_edit::Document, ErrorsEmitted> {
    let input = &diagnosis.get_source(file_id)[range.clone()];
    toml_edit::Document::from_str(input).map_err(|error| {
        diagnosis.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: Some("toml".into()),
            message: error.message().to_owned(),
            labels: error
                .span()
                .map(|span| Label {
                    style: LabelStyle::Primary,
                    file_id,
                    range: range.start + span.start..range.start + span.end,
                    message: String::new(),
                })
                .into_iter()
                .collect(),
            notes: vec![],
        });
        ErrorsEmitted
    })
}
