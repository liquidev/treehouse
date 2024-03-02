use axum::{
    http::{header::CONTENT_TYPE, Response},
    Router,
};

#[derive(Debug, Clone, Copy)]
pub struct DisableLiveReload;

pub fn live_reload(router: Router) -> Router {
    router.layer(tower_livereload::LiveReloadLayer::new().response_predicate(
        |response: &Response<_>| {
            let is_html = response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.starts_with("text/html"));
            let is_disabled = response.extensions().get::<DisableLiveReload>().is_some();
            is_html && !is_disabled
        },
    ))
}
