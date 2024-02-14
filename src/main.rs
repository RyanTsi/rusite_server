use std::{fs, str::from_utf8};

use axum::{
    response::{Html, IntoResponse, Response}, routing::get, Json, Router
};
use rusite_server::fallback::routers_static;
use serde_json::json;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(blog_router())
        .fallback_service(routers_static());
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8216").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn blog_router() -> Router {
    Router::new()
        .route("/blog", get(handler_blog))

}

async fn handler_blog() -> impl IntoResponse {
    let s = fs::read("public/1666251518.json").expect("->> not found !! ");
    let s = from_utf8(&s).unwrap();
    Json(s.to_string())
}