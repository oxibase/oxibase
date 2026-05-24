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
    Router,
};
use http_body_util::BodyExt;
use oxibase::api::Database;
use oxibase::server::create_router;
use tower::ServiceExt;

async fn get_json(app: &Router, path: &str) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_meta_schemas() {
    let db = Database::open_in_memory().unwrap();
    let app = create_router(db);

    let json = get_json(&app, "/api/meta/schemas").await;
    assert!(json.is_array());
}

#[tokio::test]
async fn test_meta_tables() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE test_table (id INTEGER)", ())
        .unwrap();
    let app = create_router(db);

    let json = get_json(&app, "/api/meta/tables").await;
    assert!(json.is_array());
    let arr = json.as_array().unwrap();

    // Check if test_table exists in the response
    let found = arr
        .iter()
        .any(|val| val.get("table_name").and_then(|v| v.as_str()) == Some("test_table"));
    assert!(found, "Table not found in metadata output");
}

#[tokio::test]
async fn test_meta_columns() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE test_table (test_col INTEGER)", ())
        .unwrap();
    let app = create_router(db);

    let json = get_json(&app, "/api/meta/columns").await;
    assert!(json.is_array());
    let arr = json.as_array().unwrap();

    let found = arr.iter().any(|val| {
        val.get("table_name").and_then(|v| v.as_str()) == Some("test_table")
            && val.get("column_name").and_then(|v| v.as_str()) == Some("test_col")
    });
    assert!(found, "Column not found in metadata output");
}

#[tokio::test]
async fn test_create_drop_table_api() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE SCHEMA IF NOT EXISTS public", ())
        .unwrap();
    let app = create_router(db.clone());

    // Create table via API
    let payload = serde_json::json!({
        "name": "new_api_table",
        "columns": [
            { "name": "id", "data_type": "INTEGER", "is_nullable": false },
            { "name": "data", "data_type": "TEXT" }
        ]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/meta/tables")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    println!("Response: {:?}", json);
    assert_eq!(status, StatusCode::CREATED);

    // Verify it exists
    assert!(db.table_exists("new_api_table").unwrap_or(false));

    // Drop table via API
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/meta/tables/public.new_api_table")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify it was dropped
    assert!(!db.table_exists("new_api_table").unwrap_or(false));
}
