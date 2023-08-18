mod html;

use axum::Router;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, LabelStyle, Severity},
    files::SimpleFile,
    term::termcolor::{ColorChoice, StandardStream},
};
use copy_dir::copy_dir;
use handlebars::Handlebars;
use html::tree::branches_to_html;
use serde::Serialize;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use treehouse_format::{ast::Roots, pull::Parser};

#[derive(Serialize)]
pub struct TemplateData {
    pub tree: String,
}

fn regenerate() -> anyhow::Result<()> {
    let _ = std::fs::remove_dir_all("target/site");
    std::fs::create_dir_all("target/site")?;

    copy_dir("static", "target/site/static")?;

    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("template/index.hbs", "template/index.hbs")?;

    let root_file = std::fs::read_to_string("content/tree/root.tree")?;
    let parse_result = Roots::parse(&mut Parser {
        input: &root_file,
        position: 0,
    });

    match parse_result {
        Ok(roots) => {
            let mut tree = String::new();
            branches_to_html(&mut tree, &roots.branches, &root_file);

            let index_html = handlebars.render("template/index.hbs", &TemplateData { tree })?;

            std::fs::write("target/site/index.html", index_html)?;
        }
        Err(error) => {
            let writer = StandardStream::stderr(ColorChoice::Auto);
            let config = codespan_reporting::term::Config::default();
            let files = SimpleFile::new("root.tree", &root_file);
            let diagnostic = Diagnostic {
                severity: Severity::Error,
                code: None,
                message: error.kind.to_string(),
                labels: vec![Label {
                    style: LabelStyle::Primary,
                    file_id: (),
                    range: error.range,
                    message: String::new(),
                }],
                notes: vec![],
            };
            codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diagnostic)?;
        }
    }

    Ok(())
}

fn regenerate_or_report_error() {
    eprintln!("regenerating");

    match regenerate() {
        Ok(_) => (),
        Err(error) => eprintln!("error: {error:?}"),
    }
}

async fn web_server() -> anyhow::Result<()> {
    let app = Router::new().nest_service("/", ServeDir::new("target/site"));

    #[cfg(debug_assertions)]
    let app = app.layer(LiveReloadLayer::new());

    Ok(axum::Server::bind(&([0, 0, 0, 0], 8080).into())
        .serve(app.into_make_service())
        .await?)
}

async fn fallible_main() -> anyhow::Result<()> {
    regenerate_or_report_error();

    web_server().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match fallible_main().await {
        Ok(_) => (),
        Err(error) => eprintln!("fatal: {error:?}"),
    }

    Ok(())
}
