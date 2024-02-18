use std::{fs, str::from_utf8 };

use axum::{
    extract::Path,
    http::{Method, Request},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json,
    Router
};
use rusite_server::{fallback::routers_static, model::ModelController, web};
use tower_cookies::CookieManagerLayer;
pub use rusite_server::error::{Error, Result};

use tower_http::cors::{CorsLayer, any};


#[tokio::main]
async fn main() -> Result<()> {
    
    // 初始化模型控制器
    let mc = ModelController::new().await?;


    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(blog_router())
        .merge(web::routes_login::routes())
        .nest("/api", web::routes_tickets::routes(mc.clone()))
        .layer(middleware::map_response(main_response_mapper))
        // .layer(CorsLayer::new()
        //             .allow_methods(vec![Method::GET, Method::POST])
        //             .allow_origin(any()))
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