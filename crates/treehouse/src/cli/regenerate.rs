use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use axum::Router;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, LabelStyle, Severity},
    files::Files as _,
};
use copy_dir::copy_dir;
use handlebars::Handlebars;
use log::{debug, info};
use serde::Serialize;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use walkdir::WalkDir;

use crate::{
    cli::parse::parse_tree_with_diagnostics, html::tree::branches_to_html, tree::SemaRoots,
};

use crate::state::{FileId, Treehouse};

#[derive(Default)]
struct Generator {
    tree_files: Vec<PathBuf>,
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
        treehouse: &mut Treehouse,
        name: &str,
        path: &Path,
    ) -> anyhow::Result<FileId> {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read template file {path:?}"))?;
        let file_id = treehouse.add_file(path.to_string_lossy().into_owned(), None, source);
        let source = treehouse.source(file_id);
        if let Err(error) = handlebars.register_template_string(name, source) {
            Self::wrangle_handlebars_error_into_diagnostic(
                treehouse,
                file_id,
                error.line_no,
                error.column_no,
                error.reason().to_string(),
            )?;
        }
        Ok(file_id)
    }

    fn wrangle_handlebars_error_into_diagnostic(
        treehouse: &mut Treehouse,
        file_id: FileId,
        line: Option<usize>,
        column: Option<usize>,
        message: String,
    ) -> anyhow::Result<()> {
        if let (Some(line), Some(column)) = (line, column) {
            let line_range = treehouse
                .files
                .line_range(file_id, line)
                .expect("file was added to the list");
            treehouse.diagnostics.push(Diagnostic {
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
            let file = treehouse.filename(file_id);
            bail!("template error in {file}: {message}");
        }
        Ok(())
    }

    fn generate_all_files(&self, dirs: &Dirs<'_>) -> anyhow::Result<Treehouse> {
        let mut treehouse = Treehouse::new();

        let mut handlebars = Handlebars::new();
        let tree_template = Self::register_template(
            &mut handlebars,
            &mut treehouse,
            "tree",
            &dirs.template_dir.join("tree.hbs"),
        )?;

        for path in &self.tree_files {
            let utf8_filename = path.to_string_lossy();

            let tree_path = path.strip_prefix(dirs.content_dir).unwrap_or(path);
            let target_path = if tree_path == OsStr::new("index.tree") {
                dirs.target_dir.join("index.html")
            } else {
                dirs.target_dir.join(tree_path).with_extension("html")
            };
            debug!("generating: {path:?} -> {target_path:?}");

            let source = match std::fs::read_to_string(path) {
                Ok(source) => source,
                Err(error) => {
                    treehouse.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: None,
                        message: format!("{utf8_filename}: cannot read file: {error}"),
                        labels: vec![],
                        notes: vec![],
                    });
                    continue;
                }
            };
            let file_id = treehouse.add_file(
                utf8_filename.into_owned(),
                Some(tree_path.with_extension("").to_string_lossy().into_owned()),
                source,
            );

            if let Ok(roots) = parse_tree_with_diagnostics(&mut treehouse, file_id) {
                let roots = SemaRoots::from_roots(&mut treehouse, file_id, roots);

                let mut tree = String::new();
                branches_to_html(&mut tree, &mut treehouse, file_id, &roots.branches);

                let template_data = TemplateData { tree };
                let templated_html = match handlebars.render("tree", &template_data) {
                    Ok(html) => html,
                    Err(error) => {
                        Self::wrangle_handlebars_error_into_diagnostic(
                            &mut treehouse,
                            tree_template,
                            error.line_no,
                            error.column_no,
                            error.desc,
                        )?;
                        continue;
                    }
                };

                std::fs::create_dir_all(
                    target_path
                        .parent()
                        .expect("there should be a parent directory to generate files into"),
                )?;
                std::fs::write(target_path, templated_html)?;
            }
        }

        Ok(treehouse)
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
    let treehouse = generator.generate_all_files(dirs)?;

    treehouse.report_diagnostics()?;

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
