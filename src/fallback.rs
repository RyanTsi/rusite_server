use tower_http::services::ServeDir;
use axum::{routing::get_service, Router};

pub fn routers_static() -> Router {
    Router::new()
        .nest_service("/", get_service(ServeDir::new("./")))
}