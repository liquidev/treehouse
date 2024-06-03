use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};

pub fn router<S>() -> Router<S> {
    if cfg!(debug_assertions) {
        Router::new()
            .route("/read-file/*path", get(read_file))
            .route("/write-file/*path", post(write_file))
            .with_state(())
    } else {
        Router::new().with_state(())
    }
}

async fn read_file(Path(path): Path<String>) -> Response {
    match std::fs::read(path) {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

async fn write_file(Path(path): Path<String>, data: Bytes) -> Response {
    match std::fs::write(path, data) {
        Ok(()) => (StatusCode::OK, String::new()).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}
