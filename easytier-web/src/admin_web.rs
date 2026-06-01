use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "admin-frontend/"]
struct AdminAssets;

pub fn build_router() -> Router {
    Router::new()
        .route("/admin", get(handle_admin_index))
        .route("/admin/*path", get(handle_admin_static))
}

async fn handle_admin_index() -> impl IntoResponse {
    serve_asset("index.html").await
}

async fn handle_admin_static(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    if path.is_empty() {
        return serve_asset("index.html").await;
    }
    serve_asset(&path).await
}

async fn serve_asset(path: &str) -> Response {
    match AdminAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.to_string())], content.data.to_vec()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}
