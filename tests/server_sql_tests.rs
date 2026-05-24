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
use tower::ServiceExt;

#[tokio::test]
async fn test_sql_endpoint_select() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE sql_test (val INTEGER)", ())
        .unwrap();
    db.execute("INSERT INTO sql_test VALUES (42)", ()).unwrap();
    let app = create_router(db);

    let payload = serde_json::json!({
        "query": "SELECT val FROM sql_test"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/sql")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let columns = json.get("columns").unwrap().as_array().unwrap();
    assert_eq!(columns.len(), 1);
    assert_eq!(columns[0].as_str().unwrap(), "val");

    let rows = json.get("rows").unwrap().as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get("val").unwrap().as_i64().unwrap(), 42);
}

#[tokio::test]
async fn test_sql_endpoint_execute() {
    let db = Database::open_in_memory().unwrap();
    let app = create_router(db.clone());

    let payload = serde_json::json!({
        "query": "CREATE TABLE sql_exec_test (id INTEGER)"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/sql")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Non-returning query should have rows_affected
    let affected = json.get("rows_affected").unwrap().as_i64().unwrap();
    assert_eq!(affected, 0); // CREATE TABLE returns 0 affected rows

    // Verify side effect
    assert!(db.table_exists("sql_exec_test").unwrap_or(false));
}
