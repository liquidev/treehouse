use axum::{http::Response, Router};

#[derive(Debug, Clone, Copy)]
pub struct DisableLiveReload;

pub fn live_reload(router: Router) -> Router {
    router.layer(tower_livereload::LiveReloadLayer::new().response_predicate(
        |response: &Response<_>| {
            let is_html = response
                .headers()
                .get("Content-Type")
                .is_some_and(|v| v == "text/html");
            let is_disabled = response.extensions().get::<DisableLiveReload>().is_some();
            is_html && !is_disabled
        },
    ))
}
