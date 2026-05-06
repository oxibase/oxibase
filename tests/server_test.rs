// Copyright 2025 Stoolap Contributors
// Copyright 2025 Oxibase Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License.
// See the License for the specific language governing permissions and
// limitations under the License.

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

#[tokio::test]
async fn test_auto_api_edge_cases_and_errors() {
    let db = Database::open("memory://").unwrap();
    db.execute("CREATE TABLE complex_types (id INT, is_active BOOLEAN, score FLOAT, tags JSON, label TEXT)", ())
        .unwrap();
    let app = create_router(db);

    // 1. 404s for missing tables
    let req = Request::builder().uri("/api/nope").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let req = Request::builder().method("POST").uri("/api/nope").header("content-type", "application/json").body(Body::from("{}")).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let req = Request::builder().method("PATCH").uri("/api/nope?id=eq.1").header("content-type", "application/json").body(Body::from("{}")).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let req = Request::builder().method("DELETE").uri("/api/nope?id=eq.1").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // 2. 400s for empty payloads on POST/PATCH
    let req = Request::builder().method("POST").uri("/api/complex_types").header("content-type", "application/json").body(Body::from("{}")).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let req = Request::builder().method("PATCH").uri("/api/complex_types?id=eq.1").header("content-type", "application/json").body(Body::from("{}")).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 3. 400s for missing eq. filters on PATCH/DELETE
    let req = Request::builder().method("PATCH").uri("/api/complex_types").header("content-type", "application/json").body(Body::from(json!({"label": "foo"}).to_string())).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let req = Request::builder().method("DELETE").uri("/api/complex_types").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 4. JSON type conversions (inserting null, bool, float, array)
    let req = Request::builder()
        .method("POST")
        .uri("/api/complex_types")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "id": 1,
            "is_active": true,
            "score": 42.5,
            "tags": ["a", "b"],
            "label": null
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // Verify type conversions via GET
    let req = Request::builder().uri("/api/complex_types").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let body = get_json_response(res).await;
    assert_eq!(body[0]["is_active"], true);
    assert_eq!(body[0]["score"], 42.5);
    assert_eq!(body[0]["tags"], "[\"a\",\"b\"]"); // currently stores complex json as strings
    assert_eq!(body[0]["label"], Value::Null);

    // 5. 500s for DB execution errors
    // Bad select column
    let req = Request::builder().uri("/api/complex_types?select=bad_col").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Type mismatch on insert (insert string to int column)
    let req = Request::builder()
        .method("POST")
        .uri("/api/complex_types")
        .header("content-type", "application/json")
        .body(Body::from(json!({"id": "not_an_int"}).to_string()))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
