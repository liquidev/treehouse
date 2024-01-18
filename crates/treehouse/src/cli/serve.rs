use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{RawQuery, State},
    response::Html,
    routing::get,
    Router,
};
use log::{error, info};
use pulldown_cmark::escape::escape_html;
use tower_http::services::ServeDir;

use crate::state::{Source, Treehouse};

use super::Paths;

struct SystemPages {
    four_oh_four: String,
    b_docs: String,
}

struct Server {
    treehouse: Treehouse,
    target_dir: PathBuf,
    system_pages: SystemPages,
}

pub async fn serve(treehouse: Treehouse, paths: &Paths<'_>, port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .nest_service("/", ServeDir::new(paths.target_dir))
        .route("/b", get(branch))
        .with_state(Arc::new(Server {
            treehouse,
            target_dir: paths.target_dir.to_owned(),
            system_pages: SystemPages {
                four_oh_four: std::fs::read_to_string(paths.target_dir.join("_treehouse/404.html"))
                    .context("cannot read 404 page")?,
                b_docs: std::fs::read_to_string(paths.target_dir.join("_treehouse/b.html"))
                    .context("cannot read /b documentation page")?,
            },
        }));

    #[cfg(debug_assertions)]
    let app = app.layer(tower_livereload::LiveReloadLayer::new());

    info!("serving on port {port}");
    Ok(axum::Server::bind(&([0, 0, 0, 0], port).into())
        .serve(app.into_make_service())
        .await?)
}

async fn branch(RawQuery(named_id): RawQuery, State(state): State<Arc<Server>>) -> Html<String> {
    if let Some(named_id) = named_id {
        if let Some(&branch_id) = state.treehouse.branches_by_named_id.get(&named_id) {
            let branch = state.treehouse.tree.branch(branch_id);
            if let Source::Tree { input, tree_path } = state.treehouse.source(branch.file_id) {
                let file_path = state.target_dir.join(format!("{tree_path}.html"));
                match std::fs::read_to_string(&file_path) {
                    Ok(content) => {
                        let branch_markdown_content = input[branch.content.clone()].trim();
                        let mut per_page_metadata =
                            String::from("<meta property=\"og:description\" content=\"");
                        escape_html(&mut per_page_metadata, branch_markdown_content).unwrap();
                        per_page_metadata.push_str("\">");

                        const PER_PAGE_METADATA_REPLACEMENT_STRING: &str = "<!-- treehouse-ca37057a-cff5-45b3-8415-3b02dbf6c799-per-branch-metadata -->";
                        return Html(content.replacen(
                            PER_PAGE_METADATA_REPLACEMENT_STRING,
                            &per_page_metadata,
                            // Replace one under the assumption that it appears in all pages.
                            1,
                        ));
                    }
                    Err(e) => {
                        error!("error while reading file {file_path:?}: {e:?}");
                    }
                }
            }
        }

        Html(state.system_pages.four_oh_four.clone())
    } else {
        Html(state.system_pages.b_docs.clone())
    }
}
