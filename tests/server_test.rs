#![cfg(feature = "server")]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use oxibase::api::Database;
use oxibase::server::create_router;
use serde_json::{json, Value};
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

async fn setup_db() -> Database {
    let db = Database::open("memory://").unwrap();
    db.execute("CREATE TABLE users (id INT, name TEXT)", ())
        .unwrap();
    db
}

async fn get_json_response(response: axum::response::Response) -> Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_auto_api_crud_flow() {
    let db = setup_db().await;
    let app = create_router(db);

    // 1. GET /api/users (Empty)
    let req = Request::builder()
        .uri("/api/users")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!([]));

    // 2. POST /api/users (Insert)
    let req = Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "id": 1, "name": "Alice" }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = get_json_response(res).await;
    assert_eq!(body, json!({ "rows_affected": 1 }));

    // 3. GET /api/users (Verify Insert)
    let req = Request::builder()
        .uri("/api/users")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!([{"id": 1, "name": "Alice"}]));

    // 4. PATCH /api/users (Update)
    let req = Request::builder()
        .method("PATCH")
        .uri("/api/users?id=eq.1")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "name": "Alice Bob" }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!({ "rows_affected": 1 }));

    // 5. GET /api/users (Verify Update)
    let req = Request::builder()
        .uri("/api/users?id=eq.1")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!([{"id": 1, "name": "Alice Bob"}]));

    // 6. DELETE /api/users
    let req = Request::builder()
        .method("DELETE")
        .uri("/api/users?id=eq.1")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!({ "rows_affected": 1 }));

    // 7. GET /api/users (Verify Delete)
    let req = Request::builder()
        .uri("/api/users")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!([]));
}
