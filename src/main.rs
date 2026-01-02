use std::collections::HashMap;
use axum::{serve, Router};
use axum::extract::{Path, Query, Request};
use axum::routing::{get, post};
use axum_test::TestServer;
use http::{HeaderMap, Method, Uri};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    serve(listener, app).await.unwrap();
}

#[tokio::test]
async fn test_axum() {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/").await;

    response.assert_status_ok();
    response.assert_text("Hello, World!");
}

#[tokio::test]
async fn test_method_routing() {
    async fn hello_world() -> String {
        "Hello, World!".to_string()
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .route("/post", post(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello, World!");

    let response = server.post("/post").await;
    response.assert_status_ok();
    response.assert_text("Hello, World!");
}



#[tokio::test]
async fn test_request() {
    async fn hello_world(request: Request) -> String {
        format!("Hello, {}!", request.method())
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .route("/post", post(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello, GET!");

}


#[tokio::test]
async fn test_uri() {
    async fn route(uri: Uri, method: Method) -> String {
        format!("Hello, {} {}!", method.as_str(), uri.path())
    }

    let app = Router::new().route("/uri", get(route));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/uri").await;
    response.assert_status_ok();
    response.assert_text("Hello, GET /uri!");

}

#[tokio::test]
async fn test_query() {
    async fn route(Query(params): Query<HashMap<String, String>>) -> String {
        format!("Hello, {}!", params.get("name").unwrap())
    }

    let app = Router::new().route("/query", get(route));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/query").add_query_param("name", "Fadel").await;
    response.assert_status_ok();
    response.assert_text("Hello, Fadel!");

}



#[tokio::test]
async fn test_header() {
    async fn route(headers: HeaderMap) -> String {
        let name = headers["name"].to_str().unwrap();
        format!("Hello, {}!", name)
    }

    let app = Router::new().route("/get", get(route));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").add_header("name", "Fadel").await;
    response.assert_status_ok();
    response.assert_text("Hello, Fadel!");

}

#[tokio::test]
async fn test_path_parameter() {
    async fn hello_world(Path((id, id_category)): Path<(String, String)>) -> String {
        format!("Product {}, Category {}", id, id_category)
    }

    let app = Router::new().route("/products/{id}/categories/{id_category}", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/products/1/categories/2").await;
    response.assert_status_ok();
    response.assert_text("Product 1, Category 2");

}


