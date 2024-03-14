use axum::{
    extract::{Path, Query, State}, http::Method, middleware, response::{IntoResponse, Response}, routing::get, Json, Router
};
use pulldown_cmark::Tag;
use rusite_server::fallback::routers_static;
use serde::Deserialize;
use sqlx::{MySql, Pool};
use tower_cookies::CookieManagerLayer;
pub use rusite_server::error::{Error, Result};

use tower_http::cors::{CorsLayer, any};

use push_server::dbops::{
    tables_ops::{ query_essay_content, query_essay_info, query_essays_info_from_tag },
    utils::build_pool
};

#[derive(Clone)]
struct AppState {
    db: Pool<MySql>,
}

impl AppState {
    fn new(db: Pool<MySql>) -> Self {
        Self {db}
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    
    dotenv::dotenv().ok();

    let pool = build_pool().await.unwrap();

    let state = AppState::new(pool);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest ("/api", api_route(state))
        .layer(middleware::map_response(main_response_mapper))
        .layer(CorsLayer::new()
                    .allow_methods(vec![Method::GET, Method::POST])
                    .allow_origin(any()))
        .layer(CookieManagerLayer::new())
        .fallback_service(routers_static());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8216").await.unwrap();
    println!("->> LISTENING on {:?}\n", listener.local_addr());
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}

async fn main_response_mapper(res: Response) -> Response {
    println!("->> {:<12} - main_response_mapper\n", "RES_MAPPER");
    res
}

fn api_route(state: AppState) -> Router {
    Router::new()
        .nest("/blog", blog_route(state))
}
    
fn blog_route(state: AppState) -> Router {
    Router::new()
        .route("/", get(handler_blog_info_list))
        .route("/querytag", get(handler_blog_tag_query))
        .route("/querycategory", get(handler_blog_category_query))
        .route("/:eid", get(handler_blog_content))
        .with_state(state)
}

async fn handler_blog_info_list(
    State(state): State<AppState>,
) -> impl IntoResponse {
    println!("->> {:<12} - handler_blog_info_list", "HANDLER");
    let pool = state.db;
    let res = query_essay_info(&pool).await.unwrap();
    Json(res)
}

async fn handler_blog_content(
    Path(eid): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    println!("->> {:<12} - handler_blog_content", "HANDLER");
    let pool = &state.db;
    let res = query_essay_content(pool, &eid).await.unwrap();
    res.unwrap_or(Default::default())
}

#[derive(Debug, Deserialize)]
struct TagParams {
    tag: Option<String>
}

#[derive(Debug, Deserialize)]
struct CategoryParams {
    category: Option<String>
}

async fn handler_blog_tag_query(
    Query(params): Query<TagParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    println!("->> {:<12} - handler_blog_tag_query", "HANDLER");
    let pool = &state.db;
    let tag = params.tag.as_deref().unwrap_or_default();
    let res = query_essays_info_from_tag(pool, tag).await.unwrap();
    Json(res)
}

async fn handler_blog_category_query(
    Query(params): Query<CategoryParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    println!("->> {:<12} - handler_blog_category_query", "HANDLER");
    let pool = &state.db;
    let category = params.category.as_deref().unwrap_or_default();
    let res = query_essays_info_from_tag(pool, category).await.unwrap();
    Json(res)
}