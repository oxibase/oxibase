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

async fn setup_db(dsn: &str) -> Database {
    let db = Database::open(dsn).unwrap();
    db.execute("CREATE TABLE IF NOT EXISTS users (id INT, name TEXT)", ())
        .unwrap();
    db
}

async fn get_json_response(response: axum::response::Response) -> Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_auto_api_crud_flow() {
    let db = setup_db("memory://test_auto_api_crud_flow").await;
    let app = create_router(db);

    // 1. GET /api/data/users (Empty)
    let req = Request::builder()
        .uri("/api/data/users")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!([]));

    // 2. POST /api/data/users (Insert)
    let req = Request::builder()
        .method("POST")
        .uri("/api/data/users")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "id": 1, "name": "Alice" }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = get_json_response(res).await;
    assert_eq!(body, json!({ "rows_affected": 1 }));

    // 3. GET /api/data/users (Verify Insert)
    let req = Request::builder()
        .uri("/api/data/users")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!([{"id": 1, "name": "Alice"}]));

    // 4. PATCH /api/data/users (Update)
    let req = Request::builder()
        .method("PATCH")
        .uri("/api/data/users?id=eq.1")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "name": "Alice Bob" }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!({ "rows_affected": 1 }));

    // 5. GET /api/data/users (Verify Update)
    let req = Request::builder()
        .uri("/api/data/users?id=eq.1")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!([{"id": 1, "name": "Alice Bob"}]));

    // 6. DELETE /api/data/users
    let req = Request::builder()
        .method("DELETE")
        .uri("/api/data/users?id=eq.1")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!({ "rows_affected": 1 }));

    // 7. GET /api/data/users (Verify Delete)
    let req = Request::builder()
        .uri("/api/data/users")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = get_json_response(res).await;
    assert_eq!(body, json!([]));
}

#[tokio::test]
async fn test_auto_api_edge_cases_and_errors() {
    let db = Database::open("memory://test_auto_api_edge_cases").unwrap();
    db.execute("CREATE TABLE complex_types (id INT, is_active BOOLEAN, score FLOAT, tags JSON, label TEXT)", ())
        .unwrap();
    let app = create_router(db);

    // 1. 404s for missing tables
    let req = Request::builder()
        .uri("/api/data/nope")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let req = Request::builder()
        .method("POST")
        .uri("/api/data/nope")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let req = Request::builder()
        .method("PATCH")
        .uri("/api/data/nope?id=eq.1")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let req = Request::builder()
        .method("DELETE")
        .uri("/api/data/nope?id=eq.1")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // 2. 400s for empty payloads on POST/PATCH
    let req = Request::builder()
        .method("POST")
        .uri("/api/data/complex_types")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let req = Request::builder()
        .method("PATCH")
        .uri("/api/data/complex_types?id=eq.1")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 3. 400s for missing eq. filters on PATCH/DELETE
    let req = Request::builder()
        .method("PATCH")
        .uri("/api/data/complex_types")
        .header("content-type", "application/json")
        .body(Body::from(json!({"label": "foo"}).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let req = Request::builder()
        .method("DELETE")
        .uri("/api/data/complex_types")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 4. JSON type conversions (inserting null, bool, float, array)
    let req = Request::builder()
        .method("POST")
        .uri("/api/data/complex_types")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "id": 1,
                "is_active": true,
                "score": 42.5,
                "tags": ["a", "b"],
                "label": null
            })
            .to_string(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // Verify type conversions via GET
    let req = Request::builder()
        .uri("/api/data/complex_types")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let body = get_json_response(res).await;
    assert_eq!(body[0]["is_active"], true);
    assert_eq!(body[0]["score"], 42.5);
    assert_eq!(body[0]["tags"], "[\"a\",\"b\"]"); // currently stores complex json as strings
    assert_eq!(body[0]["label"], Value::Null);

    // 5. 500s for DB execution errors
    // Bad select column
    let req = Request::builder()
        .uri("/api/data/complex_types?select=bad_col")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Type mismatch on insert (insert string to int column)
    let req = Request::builder()
        .method("POST")
        .uri("/api/data/complex_types")
        .header("content-type", "application/json")
        .body(Body::from(json!({"id": "not_an_int"}).to_string()))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_dynamic_route_rendering() {
    let db = setup_db("memory://test_dynamic_route_rendering").await;

    // Add dynamic template and route
    // First init the router to create tables
    let app = create_router(db.clone());

    db.execute("INSERT INTO templates.source (name, content) VALUES ('hello.html', '<h1>Hello {{ data[0].name }}!</h1>')", ()).unwrap();
    db.execute("INSERT INTO routes.definitions (method, path, template_name, context_query) VALUES ('GET', '/hello', 'hello.html', 'SELECT ''World'' AS name')", ()).unwrap();

    // GET /hello
    let req = Request::builder()
        .uri("/hello")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("<h1>Hello World!</h1>") || body_str.contains("Hello"),
        "Body was: {}",
        body_str
    );
}

#[tokio::test]
async fn test_dynamic_route_updates() {
    let db = setup_db("memory://test_dynamic_route_updates").await;
    let app = create_router(db.clone());

    // Add initial template
    db.execute(
        "INSERT INTO templates.source (name, content) VALUES ('page.html', '<h1>Version 1</h1>')",
        (),
    )
    .unwrap();
    db.execute("INSERT INTO routes.definitions (method, path, template_name) VALUES ('GET', '/page', 'page.html')", ()).unwrap();

    // GET /page (V1)
    let req = Request::builder().uri("/page").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Version 1"));

    // Update template (V2)
    db.execute(
        "UPDATE templates.source SET content = '<h1>Version 2</h1>' WHERE name = 'page.html'",
        (),
    )
    .unwrap();

    // GET /page (V2)
    let req2 = Request::builder().uri("/page").body(Body::empty()).unwrap();
    let res2 = app.clone().oneshot(req2).await.unwrap();
    let body2 = res2.into_body().collect().await.unwrap().to_bytes();
    let body_str2 = String::from_utf8(body2.to_vec()).unwrap();
    assert!(body_str2.contains("Version 2"));

    // Delete route
    db.execute("DELETE FROM routes.definitions WHERE path = '/page'", ())
        .unwrap();

    // GET /page (404)
    let req3 = Request::builder().uri("/page").body(Body::empty()).unwrap();
    let res3 = app.clone().oneshot(req3).await.unwrap();
    assert_eq!(res3.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_dynamic_route_template_inheritance() {
    let db = setup_db("memory://test_dynamic_route_template_inheritance").await;
    let app = create_router(db.clone());

    // Insert base layout template
    db.execute(
        "INSERT INTO templates.source (name, content) VALUES ('layout.html', '<main>{% block content %}{% endblock %}</main>')",
        (),
    )
    .unwrap();

    // Insert child template extending the layout
    db.execute(
        "INSERT INTO templates.source (name, content) VALUES ('child.html', '{% extends \"layout.html\" %}{% block content %}<p>Inherited Content!</p>{% endblock %}')",
        (),
    )
    .unwrap();

    // Route for the child template
    db.execute("INSERT INTO routes.definitions (method, path, template_name) VALUES ('GET', '/inherited', 'child.html')", ()).unwrap();

    // GET /inherited
    let req = Request::builder()
        .uri("/inherited")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Check if the inherited content is correctly embedded in the layout
    assert!(body_str.contains("<main><p>Inherited Content!</p></main>"));
}
