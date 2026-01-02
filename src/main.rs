use std::collections::HashMap;
use axum::{serve, Json, Router};
use axum::body::Body;
use axum::extract::{Path, Query, Request};
use axum::extract::rejection::JsonRejection;
use axum::routing::{get, post};
use axum_test::TestServer;
use http::{HeaderMap, Method, StatusCode, Uri};
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use log::{error, Log};
use axum::response::{IntoResponse, Response};

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


#[tokio::test]
async fn test_body_string() {
    async fn hello_world(body: String) -> String {
        format!("Body {}", body)
    }

    let app = Router::new().route("/post", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/post").text("This is Body").await;
    response.assert_status_ok();
    response.assert_text("Body This is Body");

}

#[derive(Debug,Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}


#[tokio::test]
async fn test_json() {
    async fn hello_world(Json(request):Json<LoginRequest>) -> String {
        format!("Hello {}", request.username)
    }

    let app = Router::new().route("/post", get(hello_world));
    let request = LoginRequest { username: "Fadel".into(), password: "Fadel".into() };


    let server = TestServer::new(app).unwrap();
    let response = server.get("/post").json(&request).await;
    response.assert_status_ok();
    response.assert_text("Hello Fadel");

}


#[tokio::test]
async fn test_json_error() {
    async fn hello_world(payload: Result<Json<LoginRequest>, JsonRejection>) -> String {
        match payload {
            Ok(request) => {
                format!("Hello {}", request.username)
            }
            Err(error) => {
                format!("Error {:?}", error)
            }
        }
    }

    let app = Router::new().route("/post", post(hello_world));

    let request = LoginRequest {
        username: "Fadel".to_string(),
        password: "<PASSWORD>".to_string(),
    };

    let server = TestServer::new(app).unwrap();
    let response = server.post("/post").json(&request).await;
    response.assert_status_ok();
    response.assert_text("Hello Fadel");

    let response = server.post("/post").text("tidak valid").await;
    response.assert_status_ok();
    response.assert_text("Error MissingJsonContentType(MissingJsonContentType)");

}

#[tokio::test]
async fn test_response() {
    async fn hello_world(request: Request) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("X-Owner", "Eko")
            .body(Body::from(format!("Hello {}", request.method())))
            .unwrap()
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");
    response.assert_header("X-Owner", "Eko");
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginResponse {
    token: String,
}

#[tokio::test]
async fn test_response_json() {
    async fn hello_world() -> (Response<()>, Json<LoginResponse>) {
        ( Response::builder()
            .status(StatusCode::OK)
            .header("X-Owner", "Fadel")
            .body(())
            .unwrap(),
        Json(LoginResponse{
            token: "token".to_string(),
        }),
        )
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("{\"token\":\"token\"}");
    response.assert_header("X-Owner", "Fadel");
}



