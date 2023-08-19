use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use axum::Router;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, LabelStyle, Severity},
    files::{Files as _, SimpleFiles},
    term::termcolor::{ColorChoice, StandardStream},
};
use copy_dir::copy_dir;
use handlebars::Handlebars;
use log::{debug, info};
use serde::Serialize;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use treehouse_format::ast::Roots;
use walkdir::WalkDir;

use crate::html::tree::branches_to_html;

#[derive(Default)]
struct Generator {
    tree_files: Vec<PathBuf>,
}

type Files = SimpleFiles<String, String>;
type FileId = <Files as codespan_reporting::files::Files<'static>>::FileId;

pub struct Diagnosis {
    pub files: Files,
    pub diagnostics: Vec<Diagnostic<FileId>>,
}

impl Generator {
    fn add_directory_rec(&mut self, directory: &Path) -> anyhow::Result<()> {
        for entry in WalkDir::new(directory) {
            let entry = entry?;
            if entry.path().extension() == Some(OsStr::new("tree")) {
                self.tree_files.push(entry.path().to_owned());
            }
        }
        Ok(())
    }

    fn register_template(
        handlebars: &mut Handlebars<'_>,
        diagnosis: &mut Diagnosis,
        name: &str,
        path: &Path,
    ) -> anyhow::Result<FileId> {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read template file {path:?}"))?;
        let file_id = diagnosis
            .files
            .add(path.to_string_lossy().into_owned(), source);
        let file = diagnosis
            .files
            .get(file_id)
            .expect("file was just added to the list");
        let source = file.source();
        if let Err(error) = handlebars.register_template_string(name, source) {
            Self::wrangle_handlebars_error_into_diagnostic(
                diagnosis,
                file_id,
                error.line_no,
                error.column_no,
                error.reason().to_string(),
            )?;
        }
        Ok(file_id)
    }

    fn wrangle_handlebars_error_into_diagnostic(
        diagnosis: &mut Diagnosis,
        file_id: FileId,
        line: Option<usize>,
        column: Option<usize>,
        message: String,
    ) -> anyhow::Result<()> {
        if let (Some(line), Some(column)) = (line, column) {
            let line_range = diagnosis
                .files
                .line_range(file_id, line)
                .expect("file was added to the list");
            diagnosis.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: Some("template".into()),
                message,
                labels: vec![Label {
                    style: LabelStyle::Primary,
                    file_id,
                    range: line_range.start + column..line_range.start + column + 1,
                    message: String::new(),
                }],
                notes: vec![],
            })
        } else {
            let file = diagnosis
                .files
                .get(file_id)
                .expect("file should already be in the list");
            bail!("template error in {}: {message}", file.name());
        }
        Ok(())
    }

    fn generate_all_files(&self, dirs: &Dirs<'_>) -> anyhow::Result<Diagnosis> {
        let mut diagnosis = Diagnosis {
            files: Files::new(),
            diagnostics: vec![],
        };

        let mut handlebars = Handlebars::new();
        let tree_template = Self::register_template(
            &mut handlebars,
            &mut diagnosis,
            "tree",
            &dirs.template_dir.join("tree.hbs"),
        )?;

        for path in &self.tree_files {
            let utf8_filename = path.to_string_lossy();
            let target_file = path.strip_prefix(dirs.content_dir).unwrap_or(path);
            let target_path = if target_file == OsStr::new("index.tree") {
                dirs.target_dir.join("index.html")
            } else {
                dirs.target_dir.join(target_file).with_extension("html")
            };
            debug!("generating: {path:?} -> {target_path:?}");

            let source = match std::fs::read_to_string(path) {
                Ok(source) => source,
                Err(error) => {
                    diagnosis.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: None,
                        message: format!("{utf8_filename}: cannot read file: {error}"),
                        labels: vec![],
                        notes: vec![],
                    });
                    continue;
                }
            };
            let file_id = diagnosis.files.add(utf8_filename.into_owned(), source);
            let source = diagnosis
                .files
                .get(file_id)
                .expect("file was just added to the list")
                .source();

            let parse_result = Roots::parse(&mut treehouse_format::pull::Parser {
                input: source,
                position: 0,
            });

            match parse_result {
                Ok(roots) => {
                    let mut tree = String::new();
                    branches_to_html(&mut tree, &roots.branches, source);

                    let template_data = TemplateData { tree };
                    let templated_html = match handlebars.render("tree", &template_data) {
                        Ok(html) => html,
                        Err(error) => {
                            Self::wrangle_handlebars_error_into_diagnostic(
                                &mut diagnosis,
                                tree_template,
                                error.line_no,
                                error.column_no,
                                error.desc,
                            )?;
                            continue;
                        }
                    };

                    std::fs::write(target_path, templated_html)?;
                }
                Err(error) => diagnosis.diagnostics.push(Diagnostic {
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
                }),
            }
        }

        Ok(diagnosis)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dirs<'a> {
    pub target_dir: &'a Path,
    pub static_dir: &'a Path,
    pub template_dir: &'a Path,
    pub content_dir: &'a Path,
}

#[derive(Serialize)]
pub struct TemplateData {
    pub tree: String,
}

pub fn regenerate(dirs: &Dirs<'_>) -> anyhow::Result<()> {
    info!("cleaning target directory");
    let _ = std::fs::remove_dir_all(dirs.target_dir);
    std::fs::create_dir_all(dirs.target_dir)?;

    info!("copying static directory to target directory");
    copy_dir(dirs.static_dir, dirs.target_dir.join("static"))?;

    info!("generating standalone pages");
    let mut generator = Generator::default();
    generator.add_directory_rec(dirs.content_dir)?;
    let diagnosis = generator.generate_all_files(dirs)?;

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = codespan_reporting::term::Config::default();
    for diagnostic in &diagnosis.diagnostics {
        codespan_reporting::term::emit(&mut writer.lock(), &config, &diagnosis.files, diagnostic)
            .context("could not emit diagnostic")?;
    }

    Ok(())
}

pub fn regenerate_or_report_error(dirs: &Dirs<'_>) {
    info!("regenerating site content");

    match regenerate(dirs) {
        Ok(_) => (),
        Err(error) => eprintln!("error: {error:?}"),
    }
}

pub async fn web_server() -> anyhow::Result<()> {
    let app = Router::new().nest_service("/", ServeDir::new("target/site"));

    #[cfg(debug_assertions)]
    let app = app.layer(LiveReloadLayer::new());

    info!("serving on port 8080");
    Ok(axum::Server::bind(&([0, 0, 0, 0], 8080).into())
        .serve(app.into_make_service())
        .await?)
}
