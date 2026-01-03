use axum::body::{Body, Bytes};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Multipart, Path, Query, Request, State};
use axum::middleware::{Next, from_fn, map_request};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router, serve, Extension};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use axum_test::TestServer;
use axum_test::multipart::{MultipartForm, Part};
use http::{HeaderMap, Method, StatusCode, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use axum::error_handling::HandleError;
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

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[tokio::test]
async fn test_json() {
    async fn hello_world(Json(request): Json<LoginRequest>) -> String {
        format!("Hello {}", request.username)
    }

    let app = Router::new().route("/post", get(hello_world));
    let request = LoginRequest {
        username: "Fadel".into(),
        password: "Fadel".into(),
    };

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
        (
            Response::builder()
                .status(StatusCode::OK)
                .header("X-Owner", "Fadel")
                .body(())
                .unwrap(),
            Json(LoginResponse {
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

#[tokio::test]
async fn test_response_tupple() {
    async fn hello_world() -> (Response<()>, Json<LoginResponse>) {
        (
            Response::builder()
                .status(StatusCode::OK)
                .header("X-Owner", "Fadel")
                .body(())
                .unwrap(),
            Json(LoginResponse {
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

#[tokio::test]
async fn test_response_tupple3() {
    async fn hello_world() -> (StatusCode, HeaderMap, Json<LoginResponse>) {
        let mut header = HeaderMap::new();
        header.insert("X-Owner", "Fadel".parse().unwrap());

        (
            StatusCode::OK,
            header,
            Json(LoginResponse {
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

#[tokio::test]
async fn test_form() {
    async fn hello_world(Form(request): Form<LoginRequest>) -> String {
        format!("Hello {}", request.username)
    }

    let app = Router::new().route("/post", post(hello_world));

    let request = LoginRequest {
        username: "Fadel".to_string(),
        password: "<PASSWORD>".to_string(),
    };
    let server = TestServer::new(app).unwrap();
    let response = server.post("/post").form(&request).await;
    response.assert_status_ok();
    response.assert_text("Hello Fadel");
}

#[tokio::test]
async fn test_multipart() {
    async fn hello_world(mut payload: Multipart) -> String {
        let mut profile: Bytes = Bytes::new();
        let mut username: String = "".to_string();

        while let Some(field) = payload.next_field().await.unwrap() {
            if field.name().unwrap_or("") == "profile" {
                profile = field.bytes().await.unwrap()
            } else if field.name().unwrap_or("") == "username" {
                username = field.text().await.unwrap()
            }
        }

        assert!(profile.len() > 0);
        format!("Hello {}", username)
    }

    let app = Router::new().route("/post", post(hello_world));

    let request = MultipartForm::new()
        .add_text("username", "Eko")
        .add_text("password", "rahasia")
        .add_part("profile", Part::bytes(Bytes::from("Contoh")));

    let server = TestServer::new(app).unwrap();
    let response = server.post("/post").multipart(request).await;
    response.assert_status_ok();
    response.assert_text("Hello Eko");
}

#[tokio::test]
async fn test_cookie_response() {
    async fn hello_world(query: Query<HashMap<String, String>>) -> (CookieJar, String) {
        let name = query.get("name").unwrap();
        (
            CookieJar::new().add(Cookie::new("name", name.clone())),
            format!("Hello {}", name.clone()),
        )
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").add_query_param("name", "Fadel").await;
    response.assert_status_ok();
    response.assert_text("Hello Fadel");
    response.assert_header("Set-Cookie", "name=Fadel");
}

#[tokio::test]
async fn test_cookie_request() {
    async fn hello_world(cookie: CookieJar) -> String {
        let name = cookie.get("name").unwrap().value();

        format!("Hello {}", name)
    }

    let app = Router::new().route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").add_header("Cookie", "name=Fadel").await;
    response.assert_status_ok();
    response.assert_text("Hello Fadel");
}

async fn log_middleware(request: Request, next: Next) -> Response {
    println!(" Receive request {} {}", request.method(), request.uri());
    let response = next.run(request).await;
    println!("Send response {}", response.status());
    response
}

async fn request_id_middleware<T>(mut request: Request<T>) -> Request<T> {
    let request_id = "123456";
    request
        .headers_mut()
        .insert("X-Request-Id", request_id.parse().unwrap());
    request
}

#[tokio::test]
async fn test_middleware() {
    async fn hello_world(method: Method, header_map: HeaderMap) -> String {
        println!("Execute handler");
        let request_id = header_map.get("X-Request-Id").unwrap().to_str().unwrap();
        format!("Hello {} {}", method, request_id)
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .layer(map_request(request_id_middleware))
        .layer(from_fn(log_middleware));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").add_header("Cookie", "name=Eko").await;
    response.assert_status_ok();
    response.assert_text("Hello GET 123456");
}

struct AppError {
    code: i32,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.code as u16).unwrap(),
            self.message,
        )
            .into_response()
    }
}

#[tokio::test]
async fn test_error_handling() {
    async fn hello_world(method: Method) -> Result<String, AppError> {
        if method == Method::POST {
            Ok("OK".to_string())
        } else {
            Err (AppError{
                code :400,
                message: "Bad Request".to_string(),
            })
        }

    }
    let app = Router::new()
        .route("/get", get(hello_world))
    .route("/post", post(hello_world));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status(StatusCode::BAD_REQUEST);
    response.assert_text("Bad Request");

    let response = server.post("/post").await;
    response.assert_status(StatusCode::OK);
    response.assert_text("OK");

}

#[tokio::test]
async fn test_unexpected_handling() {
    async fn route(request: Request) -> Result<Response, anyhow::Error> {
        if request.method() == Method::POST {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from("OK"))?)
        } else {
            Err (anyhow::Error::msg("Bad Request"))
        }

    }

    async fn handle_error(err: anyhow::Error) -> (StatusCode, String) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal Server Error: {}", err),
            )
    }

    let route_service = tower::service_fn(route);

    let app = Router::new()
        .route_service("/get", HandleError::new(route_service, handle_error));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;

    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    response.assert_text("Internal Server Error: Bad Request");

}

struct DatabaseConfig {
    total : i32,
}


#[tokio::test]
async fn test_state_extractor() {
    let database_state = Arc::new(DatabaseConfig{total : 100});

    async fn hello_world(State(database) : State<Arc<DatabaseConfig>>) -> String {
        format!("Total {}", database.total)
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .with_state(database_state);

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Total 100");
}

#[tokio::test]
async fn test_state_extension() {
    let database_state = Arc::new(DatabaseConfig{total : 100});

    async fn hello_world(Extension(database) : Extension<Arc<DatabaseConfig>>) -> String {
        format!("Total {}", database.total)
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .layer(Extension(database_state));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Total 100");
}



#[tokio::test]
async fn test_state_closure_capture() {
    let database_state = Arc::new(DatabaseConfig { total: 100 });

    async fn hello_world(database: Arc<DatabaseConfig>) -> String {
        format!("Total {}", database.total)
    }
    let app = Router::new()
        .route("/get", get({
            let database_state = Arc::clone(&database_state);
            move || hello_world(database_state)
        }),
        )
        .layer(Extension(database_state));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Total 100");
}


#[tokio::test]
async fn test_multiple_routes() {
    async fn hello_world(method: Method) -> String {
        format!("Hello {}", method)
    }

    let first_route = Router::new().route("/first", get(hello_world));
    let second_route = Router::new().route("/second", get(hello_world));

    let app= Router::new().merge(first_route).merge(second_route);

    let server = TestServer::new(app).unwrap();
    let response = server.get("/first").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

    let response = server.get("/second").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

}


#[tokio::test]
async fn test_multiple_route_nest() {
    async fn hello_world(method: Method) -> String {
        format!("Hello {}", method)
    }

    let first_route = Router::new().route("/first", get(hello_world));
    let second_route = Router::new().route("/second", get(hello_world));

    let app= Router::new()
        .nest("/api/users", first_route)
        .nest("/api/products", second_route);

    let server = TestServer::new(app).unwrap();
    let response = server.get("/api/users/first").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

    let response = server.get("/api/products/second").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

}


#[tokio::test]
async fn test_fallback() {
    async fn hello_world(method: Method) -> String {
        format!("Hello {}", method)
    }

    async fn fallback(request: Request) -> (StatusCode, String) {
        (
            StatusCode::NOT_FOUND,
            format!("Page {} is not found", request.uri().path()),
            )
    }


    async fn not_allowed(request: Request) -> (StatusCode, String) {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            format!("Page {} is not found", request.uri().path()),
        )
    }





    let first_route = Router::new().route("/first", get(hello_world));
    let second_route = Router::new().route("/second", get(hello_world));

    let app= Router::new()
        .merge(first_route)
        .merge(second_route)
        .fallback(fallback)
        .method_not_allowed_fallback(not_allowed);

    let server = TestServer::new(app).unwrap();
    let response = server.get("/first").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

    let response = server.get("/second").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

    let response = server.get("/wrong").await;
    response.assert_status_not_found();
    response.assert_text("Page /wrong is not found");

    let response = server.post("/first").await;
    response.assert_status(StatusCode::METHOD_NOT_ALLOWED);
    response.assert_text("Page /first is not found");

}