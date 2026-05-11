use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use oxibase::{api::Database, server::create_router};

#[tokio::test]
async fn test_rpc_procedure_invocation() {
    let db = Database::open_in_memory().unwrap();

    // Create a simple procedure
    db.execute(
        r#"
        CREATE PROCEDURE test_add(a INTEGER, b INTEGER, OUT res INTEGER)
        LANGUAGE rhai AS '
            res = a + b;
        ';
        "#,
        (),
    )
    .unwrap();

    let app = create_router(db);

    // Make a request
    let req = Request::builder()
        .method("POST")
        .uri("/api/rpc/test_add")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "a": 5,
                "b": 7
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["result"], json!(12));
}

#[tokio::test]
async fn test_rpc_procedure_errors() {
    let db = Database::open_in_memory().unwrap();

    db.execute(
        r#"
        CREATE PROCEDURE test_err(amount INTEGER)
        LANGUAGE rhai AS '
            if amount < 0 {
                throw "Negative amount not allowed";
            }
        ';
        "#,
        (),
    )
    .unwrap();

    let app = create_router(db);

    // 1. Test 404 Not Found
    let req = Request::builder()
        .method("POST")
        .uri("/api/rpc/non_existent")
        .header("Content-Type", "application/json")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 2. Test 400 Missing Parameter
    let req = Request::builder()
        .method("POST")
        .uri("/api/rpc/test_err")
        .header("Content-Type", "application/json")
        .body(Body::from("{}")) // Missing 'amount'
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 3. Test 500 Execution Error
    let req = Request::builder()
        .method("POST")
        .uri("/api/rpc/test_err")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({ "amount": -5 }).to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_rpc_procedure_headers() {
    let db = Database::open_in_memory().unwrap();

    db.execute(
        r#"
        CREATE PROCEDURE check_auth(OUT res TEXT)
        LANGUAGE rhai AS '
            let token = get_http_header("Authorization");
            if token == () {
                res = "missing";
            } else {
                res = token;
            }
        ';
        "#,
        (),
    )
    .unwrap();

    let app = create_router(db);

    let req = Request::builder()
        .method("POST")
        .uri("/api/rpc/check_auth")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer token123")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status, StatusCode::OK, "Response was: {:?}", body_json);

    assert_eq!(body_json["result"], json!("Bearer token123"));
}
