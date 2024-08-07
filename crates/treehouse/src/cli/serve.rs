#[cfg(debug_assertions)]
mod live_reload;

use std::{net::Ipv4Addr, path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, Query, RawQuery, State},
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE, LOCATION},
        HeaderValue, StatusCode,
    },
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use log::{error, info};
use pulldown_cmark::escape::escape_html;
use serde::Deserialize;
use tokio::net::TcpListener;

use crate::{
    config::Config,
    state::{Source, Treehouse},
};

use super::Paths;

struct SystemPages {
    index: String,
    four_oh_four: String,
    b_docs: String,
    sandbox: String,

    navmap: String,
}

struct Server {
    config: Config,
    treehouse: Treehouse,
    target_dir: PathBuf,
    system_pages: SystemPages,
}

pub async fn serve(
    config: Config,
    treehouse: Treehouse,
    paths: &Paths<'_>,
    port: u16,
) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(index))
        .route("/*page", get(page))
        .route("/b", get(branch))
        .route("/navmap.js", get(navmap))
        .route("/sandbox", get(sandbox))
        .route("/static/*file", get(static_file))
        .fallback(get(four_oh_four))
        .with_state(Arc::new(Server {
            config,
            treehouse,
            target_dir: paths.target_dir.to_owned(),
            system_pages: SystemPages {
                index: std::fs::read_to_string(paths.target_dir.join("index.html"))
                    .context("cannot read index page")?,
                four_oh_four: std::fs::read_to_string(paths.target_dir.join("_treehouse/404.html"))
                    .context("cannot read 404 page")?,
                b_docs: std::fs::read_to_string(paths.target_dir.join("_treehouse/b.html"))
                    .context("cannot read /b documentation page")?,
                sandbox: std::fs::read_to_string(paths.target_dir.join("static/html/sandbox.html"))
                    .context("cannot read sandbox page")?,
                navmap: std::fs::read_to_string(paths.target_dir.join("navmap.js"))
                    .context("cannot read navigation map")?,
            },
        }));

    #[cfg(debug_assertions)]
    let app = live_reload::live_reload(app);

    info!("serving on port {port}");
    let listener = TcpListener::bind((Ipv4Addr::from([0u8, 0, 0, 0]), port)).await?;
    Ok(axum::serve(listener, app).await?)
}

fn get_content_type(path: &str) -> Option<&'static str> {
    match () {
        _ if path.ends_with(".html") => Some("text/html"),
        _ if path.ends_with(".js") => Some("text/javascript"),
        _ if path.ends_with(".woff2") => Some("font/woff2"),
        _ if path.ends_with(".svg") => Some("image/svg+xml"),
        _ => None,
    }
}

async fn index(State(state): State<Arc<Server>>) -> Response {
    Html(state.system_pages.index.clone()).into_response()
}

async fn navmap(State(state): State<Arc<Server>>) -> Response {
    let mut response = state.system_pages.navmap.clone().into_response();
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/javascript"));
    response
}

async fn four_oh_four(State(state): State<Arc<Server>>) -> Response {
    (
        StatusCode::NOT_FOUND,
        Html(state.system_pages.four_oh_four.clone()),
    )
        .into_response()
}

#[derive(Deserialize)]
struct StaticFileQuery {
    cache: Option<String>,
}

async fn static_file(
    Path(path): Path<String>,
    Query(query): Query<StaticFileQuery>,
    State(state): State<Arc<Server>>,
) -> Response {
    if let Ok(file) = tokio::fs::read(state.target_dir.join("static").join(&path)).await {
        let mut response = file.into_response();

        if let Some(content_type) = get_content_type(&path) {
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
        } else {
            response.headers_mut().remove(CONTENT_TYPE);
        }

        if query.cache.is_some() {
            response.headers_mut().insert(
                CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            );
        }

        response
    } else {
        four_oh_four(State(state)).await
    }
}

async fn page(Path(path): Path<String>, State(state): State<Arc<Server>>) -> Response {
    let bare_path = path.strip_suffix(".html").unwrap_or(&path);
    if let Some(redirected_path) = state.config.redirects.page.get(bare_path) {
        return (
            StatusCode::MOVED_PERMANENTLY,
            [(LOCATION, format!("{}/{redirected_path}", state.config.site))],
        )
            .into_response();
    }

    let html_path = format!("{bare_path}.html");
    if let Ok(file) = tokio::fs::read(state.target_dir.join(&*html_path)).await {
        ([(CONTENT_TYPE, "text/html")], file).into_response()
    } else {
        four_oh_four(State(state)).await
    }
}

async fn sandbox(State(state): State<Arc<Server>>) -> Response {
    // Small hack to prevent the LiveReloadLayer from injecting itself into the sandbox.
    // The sandbox is always nested under a different page, so there's no need to do that.
    let mut response = Html(state.system_pages.sandbox.clone()).into_response();
    #[cfg(debug_assertions)]
    {
        response
            .extensions_mut()
            .insert(live_reload::DisableLiveReload);
    }
    // Debounce requests a bit. There's a tendency to have very many sandboxes on a page, and
    // loading this page as many times as there are sandboxes doesn't seem like the best way to do
    // things.
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("max-age=10"));
    response
}

async fn branch(RawQuery(named_id): RawQuery, State(state): State<Arc<Server>>) -> Html<String> {
    if let Some(named_id) = named_id {
        let branch_id = state
            .treehouse
            .branches_by_named_id
            .get(&named_id)
            .copied()
            .or_else(|| state.treehouse.branch_redirects.get(&named_id).copied());
        if let Some(branch_id) = branch_id {
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
