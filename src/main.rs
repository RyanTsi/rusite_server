use std::{fs, str::from_utf8, task::Context};

use axum::{
    extract::Path, response::{Html, IntoResponse, Response}, routing::get, Json, Router
};
use rusite_server::fallback::routers_static;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(blog_router())
        .fallback_service(routers_static());
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8216").await.unwrap();
    println!("->> LISTENING on {:?}\n", listener.local_addr());
    axum::serve(listener, app).await.unwrap();
}

fn blog_router() -> Router {
    Router::new()
        .route("/blog", get(handler_blog))
        .route("/blog2/:blog_id", get(handler_blog2))
}



async fn handler_blog() -> impl IntoResponse {
    println!("->> {:<12} - handler_blog", "HANDLER");
    let s = fs::read("public/1666251518").expect("->> not found !! ");
    let s = from_utf8(&s).unwrap();
    Json(s.to_string())
}

async fn handler_blog2(Path(blog_id): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler_blog2", "HANDLER");
    let blog_path = String::from("public/") + &blog_id;
    let content = fs::read(blog_path).expect("not found !!");
    let content = from_utf8(&content).unwrap();
    Json(content.to_string())
}