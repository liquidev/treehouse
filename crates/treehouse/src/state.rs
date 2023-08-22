use std::{collections::HashMap, ops::Range};

use anyhow::Context;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, LabelStyle, Severity},
    files::SimpleFiles,
    term::termcolor::{ColorChoice, StandardStream},
};
use ulid::Ulid;

use crate::tree::{SemaBranchId, SemaTree};

pub type Files = SimpleFiles<String, String>;
pub type FileId = <Files as codespan_reporting::files::Files<'static>>::FileId;

/// Treehouse compilation context.
pub struct Treehouse {
    pub files: Files,
    pub diagnostics: Vec<Diagnostic<FileId>>,

    pub tree: SemaTree,
    pub branches_by_named_id: HashMap<String, SemaBranchId>,

    // Bit of a hack because I don't wanna write my own `Files`.
    tree_paths: Vec<Option<String>>,

    missingno_generator: ulid::Generator,
}

#[derive(Debug, Clone)]
pub struct BranchRef {
    pub html_id: String,
    pub file_id: FileId,
    pub kind_span: Range<usize>,
}

impl Treehouse {
    pub fn new() -> Self {
        Self {
            files: Files::new(),
            diagnostics: vec![],

            tree: SemaTree::default(),
            branches_by_named_id: HashMap::new(),

            tree_paths: vec![],

            missingno_generator: ulid::Generator::new(),
        }
    }

    pub fn add_file(
        &mut self,
        filename: String,
        tree_path: Option<String>,
        source: String,
    ) -> FileId {
        let id = self.files.add(filename, source);
        self.tree_paths.push(tree_path);
        id
    }

    /// Get the source code of a file, assuming it was previously registered.
    pub fn source(&self, file_id: FileId) -> &str {
        self.files
            .get(file_id)
            .expect("file should have been registered previously")
            .source()
    }

    /// Get the name of a file, assuming it was previously registered.
    pub fn filename(&self, file_id: FileId) -> &str {
        self.files
            .get(file_id)
            .expect("file should have been registered previously")
            .name()
    }

    pub fn tree_path(&self, file_id: FileId) -> Option<&str> {
        self.tree_paths[file_id].as_deref()
    }

    pub fn report_diagnostics(&self) -> anyhow::Result<()> {
        let writer = StandardStream::stderr(ColorChoice::Auto);
        let config = codespan_reporting::term::Config::default();
        for diagnostic in &self.diagnostics {
            codespan_reporting::term::emit(&mut writer.lock(), &config, &self.files, diagnostic)
                .context("could not emit diagnostic")?;
        }

        Ok(())
    }

    pub fn next_missingno(&mut self) -> Ulid {
        self.missingno_generator
            .generate()
            .expect("just how much disk space do you have?")
    }
}

pub struct TomlError {
    pub message: String,
    pub span: Option<Range<usize>>,
    pub file_id: FileId,
    pub input_range: Range<usize>,
}

pub fn toml_error_to_diagnostic(error: TomlError) -> Diagnostic<FileId> {
    Diagnostic {
        severity: Severity::Error,
        code: Some("toml".into()),
        message: error.message,
        labels: error
            .span
            .map(|span| Label {
                style: LabelStyle::Primary,
                file_id: error.file_id,
                range: error.input_range.start + span.start..error.input_range.start + span.end,
                message: String::new(),
            })
            .into_iter()
            .collect(),
        notes: vec![],
    }
}
