use anyhow::Context;
use codespan_reporting::{
    diagnostic::Diagnostic,
    files::SimpleFiles,
    term::termcolor::{ColorChoice, StandardStream},
};

pub type Files = SimpleFiles<String, String>;
pub type FileId = <Files as codespan_reporting::files::Files<'static>>::FileId;

pub struct Diagnosis {
    pub files: Files,
    pub diagnostics: Vec<Diagnostic<FileId>>,
}

impl Diagnosis {
    pub fn new() -> Self {
        Self {
            files: Files::new(),
            diagnostics: vec![],
        }
    }

    /// Get the source code of a file, assuming it was previously registered.
    pub fn get_source(&self, file_id: FileId) -> &str {
        self.files
            .get(file_id)
            .expect("file should have been registered previously")
            .source()
    }

    pub fn report(&self) -> anyhow::Result<()> {
        let writer = StandardStream::stderr(ColorChoice::Auto);
        let config = codespan_reporting::term::Config::default();
        for diagnostic in &self.diagnostics {
            codespan_reporting::term::emit(&mut writer.lock(), &config, &self.files, diagnostic)
                .context("could not emit diagnostic")?;
        }

        Ok(())
    }
}
