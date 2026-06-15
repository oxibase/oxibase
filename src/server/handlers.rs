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

// This file handles the logic for the Axum routes.
// We import required Axum and Oxibase types to be used across handlers.

use crate::api::Database;
use crate::server::template::create_env;
use crate::server::AppState;
use crate::Value;
use axum::{
    extract::{Form, Path, Query, Request, State},
    http::HeaderMap,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing_opentelemetry::OpenTelemetrySpanExt;

struct HeaderExtractor<'a>(&'a HeaderMap);

impl<'a> opentelemetry::propagation::Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

#[derive(Deserialize, Default)]
pub struct GetQueryParams {
    pub select: Option<String>,
    pub order: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    // For capturing arbitrary `col=eq.val` parameters
    #[serde(flatten)]
    pub filters: HashMap<String, String>,
}

/// Helper function to check if a table exists in the information schema
fn table_exists(db: &Database, full_table_name: &str) -> Result<bool, String> {
    let (schema_name, table_name) = if let Some(dot_pos) = full_table_name.find('.') {
        (&full_table_name[..dot_pos], &full_table_name[dot_pos + 1..])
    } else {
        ("public", full_table_name)
    };

    let query = "SELECT 1 FROM information_schema.tables WHERE table_schema = ? AND table_name = ?";
    let rows = db
        .query(
            query,
            vec![Value::text(schema_name), Value::text(table_name)],
        )
        .map_err(|e| e.to_string())?;

    // If we have at least one row, the table exists
    let mut count = 0;
    for _ in rows {
        count += 1;
    }

    Ok(count > 0)
}

/// Convert an Oxibase Value to a Serde JSON Value
pub fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Null(_) => JsonValue::Null,
        Value::Integer(i) => serde_json::json!(i),
        Value::Float(f) => serde_json::json!(f),
        Value::Text(s) => serde_json::json!(s.as_ref()),
        Value::Boolean(b) => serde_json::json!(b),
        Value::Timestamp(ts) => serde_json::json!(ts.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        Value::Json(s) => serde_json::json!(s.as_ref()),
    }
}

pub async fn get_table(
    Path(table): Path<String>,
    Query(params): Query<GetQueryParams>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let parent_cx = opentelemetry::global::get_text_map_propagator(|prop| {
        prop.extract(&HeaderExtractor(&headers))
    });
    let span = tracing::info_span!("network.request", method = "GET", path = "/api/table");
    let _ = span.set_parent(parent_cx);
    let _guard = span.enter();

    // Check if table exists
    match table_exists(&state.db, &table) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Table '{}' not found", table) })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }

    // Dynamic SELECT clause
    let select_clause = params.select.unwrap_or_else(|| "*".to_string());

    // Dynamic WHERE clause (exact match eq. operator)
    let mut where_clauses = Vec::new();
    let mut query_args = Vec::new();

    for (key, val) in &params.filters {
        if val.starts_with("eq.") {
            where_clauses.push(format!("{} = ?", key));
            let actual_val = val.trim_start_matches("eq.");
            query_args.push(Value::text(actual_val)); // Always binding as text for now
        }
    }

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Dynamic ORDER BY clause
    let order_clause = if let Some(order) = params.order {
        let mut order_parts = Vec::new();
        for part in order.split(',') {
            if part.ends_with(".desc") {
                order_parts.push(format!("{} DESC", part.trim_end_matches(".desc")));
            } else if part.ends_with(".asc") {
                order_parts.push(format!("{} ASC", part.trim_end_matches(".asc")));
            } else {
                order_parts.push(format!("{} ASC", part)); // Default to ASC
            }
        }
        format!("ORDER BY {}", order_parts.join(", "))
    } else {
        String::new()
    };

    // Dynamic LIMIT and OFFSET clauses
    let limit_clause = if let Some(limit) = params.limit {
        format!("LIMIT {}", limit)
    } else {
        String::new()
    };

    let offset_clause = if let Some(offset) = params.offset {
        format!("OFFSET {}", offset)
    } else {
        String::new()
    };

    // Construct final query
    let query = format!(
        "SELECT {} FROM {} {} {} {} {}",
        select_clause, table, where_clause, order_clause, limit_clause, offset_clause
    );
    let query = query.trim().to_string();

    let rows_result = match state.db.query(&query, query_args) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Query error: {}", e) })),
            )
                .into_response()
        }
    };

    let columns = rows_result.columns().to_vec();
    let mut all_rows = Vec::new();

    for row_res in rows_result {
        let row = match row_res {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Row error: {}", e) })),
                )
                    .into_response()
            }
        };

        let mut json_row = serde_json::Map::new();
        for (i, col_name) in columns.iter().enumerate() {
            let val = row.get_value(i).cloned().unwrap_or(Value::null_unknown());
            json_row.insert(col_name.clone(), value_to_json(&val));
        }
        all_rows.push(JsonValue::Object(json_row));
    }

    (StatusCode::OK, Json(JsonValue::Array(all_rows))).into_response()
}

/// Fallback route handler for dynamic database-driven templates
pub async fn dynamic_route_handler(
    State(state): State<AppState>,
    req: Request,
) -> impl IntoResponse {
    let method = req.method().as_str();
    let path = req.uri().path();

    // Lookup the route
    let query =
        "SELECT template_name, context_query FROM interface.routes WHERE method = ? AND path = ?";
    let rows_res = match state
        .db
        .query(query, vec![Value::text(method), Value::text(path)])
    {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error finding route: {}", e),
            )
                .into_response()
        }
    };

    let mut matching_rows = vec![];
    for r in rows_res.flatten() {
        matching_rows.push(r);
    }

    if matching_rows.is_empty() {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    let route_row = &matching_rows[0];
    let template_name = match route_row.get_value(0) {
        Some(Value::Text(s)) => s.to_string(),
        _ => return (StatusCode::INTERNAL_SERVER_ERROR, "Invalid template_name").into_response(),
    };

    let context_query = match route_row.get_value(1) {
        Some(Value::Text(s)) => Some(s.to_string()),
        _ => None,
    };

    // Build context
    let mut context = serde_json::Map::new();

    // Parse simple query parameters into context (simple decoding)
    if let Some(query_str) = req.uri().query() {
        for pair in query_str.split('&') {
            if let Some((key, val)) = pair.split_once('=') {
                let mut decoded = String::new();
                let mut chars = val.chars();
                while let Some(c) = chars.next() {
                    if c == '%' {
                        if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                            if let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                                decoded.push(byte as char);
                            } else {
                                decoded.push('%');
                                decoded.push(h1);
                                decoded.push(h2);
                            }
                        } else {
                            decoded.push('%');
                        }
                    } else if c == '+' {
                        decoded.push(' ');
                    } else {
                        decoded.push(c);
                    }
                }
                context.insert(key.to_string(), JsonValue::String(decoded));
            }
        }
    }

    if let Some(ctx_query) = context_query {
        // Run the query to fetch dynamic context
        let ctx_rows_res = match state.db.query(&ctx_query, ()) {
            Ok(res) => res,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Context query error: {}", e),
                )
                    .into_response()
            }
        };

        let cols = ctx_rows_res.columns().to_vec();
        let mut all_rows = Vec::new();

        for row_res in ctx_rows_res {
            let row = match row_res {
                Ok(r) => r,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Row error: {}", e),
                    )
                        .into_response()
                }
            };

            let mut json_row = serde_json::Map::new();
            for (i, col_name) in cols.iter().enumerate() {
                let val = row.get_value(i).cloned().unwrap_or(Value::null_unknown());
                json_row.insert(col_name.clone(), value_to_json(&val));
            }
            all_rows.push(JsonValue::Object(json_row));
        }

        context.insert("data".to_string(), JsonValue::Array(all_rows));
    }

    // Setup jinja environment
    let env = create_env(state.db.clone());

    // Render
    let tmpl = match env.get_template(&template_name) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template load error: {}", e),
            )
                .into_response()
        }
    };

    match tmpl.render(JsonValue::Object(context)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template render error: {}", e),
        )
            .into_response(),
    }
}

pub async fn insert_row(
    Path(table): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Map<String, JsonValue>>,
) -> impl IntoResponse {
    let parent_cx = opentelemetry::global::get_text_map_propagator(|prop| {
        prop.extract(&HeaderExtractor(&headers))
    });
    let span = tracing::info_span!("network.request", method = "POST", path = "/api/table");
    let _ = span.set_parent(parent_cx);
    let _guard = span.enter();

    // Check if table exists
    match table_exists(&state.db, &table) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Table '{}' not found", table) })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }

    if payload.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Empty payload" })),
        )
            .into_response();
    }

    let mut columns = Vec::new();
    let mut placeholders = Vec::new();
    let mut args = Vec::new();

    for (k, v) in payload.iter() {
        columns.push(k.clone());
        placeholders.push("?");

        let value = match v {
            JsonValue::Null => Value::null_unknown(),
            JsonValue::Bool(b) => Value::Boolean(*b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::text(n.to_string())
                }
            }
            JsonValue::String(s) => Value::text(s),
            _ => Value::text(v.to_string()), // arrays and objects as strings for now
        };
        args.push(value);
    }

    let query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        columns.join(", "),
        placeholders.join(", ")
    );

    match state.db.execute(&query, args) {
        Ok(rows_affected) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "rows_affected": rows_affected })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Insert error: {}", e) })),
        )
            .into_response(),
    }
}

pub async fn update_row(
    Path(table): Path<String>,
    Query(params): Query<GetQueryParams>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Map<String, JsonValue>>,
) -> impl IntoResponse {
    let parent_cx = opentelemetry::global::get_text_map_propagator(|prop| {
        prop.extract(&HeaderExtractor(&headers))
    });
    let span = tracing::info_span!("network.request", method = "PUT/PATCH", path = "/api/table");
    let _ = span.set_parent(parent_cx);
    let _guard = span.enter();

    // Check if table exists
    match table_exists(&state.db, &table) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Table '{}' not found", table) })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }

    if payload.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Empty payload" })),
        )
            .into_response();
    }

    // Parse filters
    let mut where_clauses = Vec::new();
    let mut args = Vec::new();

    for (key, val) in &params.filters {
        if val.starts_with("eq.") {
            where_clauses.push(format!("{} = ?", key));
            let actual_val = val.trim_start_matches("eq.");
            args.push(Value::text(actual_val));
        }
    }

    if where_clauses.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing exact match filter (e.g. ?id=eq.1)" })),
        )
            .into_response();
    }

    let mut set_clauses = Vec::new();
    let mut set_args = Vec::new();

    for (k, v) in payload.iter() {
        set_clauses.push(format!("{} = ?", k));

        let value = match v {
            JsonValue::Null => Value::null_unknown(),
            JsonValue::Bool(b) => Value::Boolean(*b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::text(n.to_string())
                }
            }
            JsonValue::String(s) => Value::text(s),
            _ => Value::text(v.to_string()),
        };
        set_args.push(value);
    }

    // Combine args: SET args first, then WHERE args
    set_args.extend(args);
    let final_args = set_args;

    let query = format!(
        "UPDATE {} SET {} WHERE {}",
        table,
        set_clauses.join(", "),
        where_clauses.join(" AND ")
    );

    match state.db.execute(&query, final_args) {
        Ok(rows_affected) => (
            StatusCode::OK,
            Json(serde_json::json!({ "rows_affected": rows_affected })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Update error: {}", e) })),
        )
            .into_response(),
    }
}

pub async fn delete_row(
    Path(table): Path<String>,
    Query(params): Query<GetQueryParams>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let parent_cx = opentelemetry::global::get_text_map_propagator(|prop| {
        prop.extract(&HeaderExtractor(&headers))
    });
    let span = tracing::info_span!("network.request", method = "DELETE", path = "/api/table");
    let _ = span.set_parent(parent_cx);
    let _guard = span.enter();

    // Check if table exists
    match table_exists(&state.db, &table) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Table '{}' not found", table) })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }

    // Parse filters
    let mut where_clauses = Vec::new();
    let mut args = Vec::new();

    for (key, val) in &params.filters {
        if val.starts_with("eq.") {
            where_clauses.push(format!("{} = ?", key));
            let actual_val = val.trim_start_matches("eq.");
            args.push(Value::text(actual_val));
        }
    }

    if where_clauses.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing exact match filter (e.g. ?id=eq.1)" })),
        )
            .into_response();
    }

    let query = format!(
        "DELETE FROM {} WHERE {}",
        table,
        where_clauses.join(" AND ")
    );

    match state.db.execute(&query, args) {
        Ok(rows_affected) => (
            StatusCode::OK,
            Json(serde_json::json!({ "rows_affected": rows_affected })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Delete error: {}", e) })),
        )
            .into_response(),
    }
}

pub async fn invoke_procedure(
    Path(procedure_name): Path<String>,
    State(state): State<AppState>,
    req: Request,
) -> impl IntoResponse {
    let (parts, body) = req.into_parts();

    let parent_cx = opentelemetry::global::get_text_map_propagator(|prop| {
        prop.extract(&HeaderExtractor(&parts.headers))
    });
    let span = tracing::info_span!("network.request", method = "POST", path = "/api/rpc");
    let _ = span.set_parent(parent_cx);
    let _guard = span.enter();

    // Extract headers into a HashMap
    let mut headers = HashMap::new();
    for (k, v) in parts.headers.iter() {
        if let Ok(value_str) = v.to_str() {
            headers.insert(k.as_str().to_string(), value_str.to_string());
        }
    }

    // Parse JSON body manually
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Failed to read body: {}", e) })),
            )
                .into_response();
        }
    };

    let payload: HashMap<String, JsonValue> = if body_bytes.is_empty() {
        HashMap::new()
    } else {
        match serde_json::from_slice(&body_bytes) {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("Invalid JSON: {}", e) })),
                )
                    .into_response();
            }
        }
    };

    let procedure_name_upper = procedure_name.to_uppercase();

    // Look up the procedure from the function registry (via DB executor)
    let executor = crate::executor::Executor::new(std::sync::Arc::clone(state.db.engine()));
    let registry = executor.function_registry();

    let procedure = match registry.get_procedure(&procedure_name_upper) {
        Some(proc) => proc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Procedure '{}' not found", procedure_name) })),
            ).into_response();
        }
    };

    // Extract IN and INOUT parameters, and build the call arguments
    let mut args = Vec::new();
    let mut param_types = Vec::new();
    let mut expected_out = Vec::new();

    for param in &procedure.parameters {
        if param.mode.eq_ignore_ascii_case("OUT") {
            expected_out.push(param.name.clone());
            // OUT parameters might not be passed in JSON, we can pass a dummy value like NULL
            args.push(crate::core::Value::null_unknown());
            param_types.push(param.data_type.clone());
            continue;
        }

        // It's IN or INOUT. Look for it in the payload
        match payload.get(&param.name) {
            Some(val) => {
                // Convert JSON value to string, or if it's already scalar, format it appropriately
                // The easiest way to handle this securely without SQL injection via formatting
                // is to use query parameters if we build a SQL string, or directly execute if we bypass parsing.
                // However, building a parameter array is cleaner.
                // Let's use `value_from_json`
                // Wait, oxibase value conversion:
                let converted_val = json_to_value(val);
                args.push(converted_val);
                param_types.push(param.data_type.clone());

                if param.mode.eq_ignore_ascii_case("INOUT") {
                    expected_out.push(param.name.clone());
                }
            }
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("Missing parameter '{}'", param.name) })),
                ).into_response();
            }
        }
    }

    // Now, we can formulate a CALL statement and execute it with parameters
    // Format: CALL proc_name($1, $2, ...)
    let placeholders: Vec<String> = (1..=args.len()).map(|i| format!("${}", i)).collect();
    let sql = format!("CALL {}({})", procedure_name, placeholders.join(", "));

    let debug_controller = std::sync::Arc::clone(&state.debug_controller);
    let db_clone = std::sync::Arc::clone(&state.db);
    let (result, stdout) = tokio::task::spawn_blocking(move || {
        crate::functions::context::with_http_headers_and_debug(
            headers,
            Some(debug_controller),
            || {
                let res = db_clone.query(&sql, args);
                let stdout_captured = crate::functions::context::get_stdout();
                (res, stdout_captured)
            },
        )
    })
    .await
    .unwrap_or_else(|e| {
        (
            Err(crate::core::Error::internal(format!("Task panic: {}", e))),
            String::new(),
        )
    });

    let result = match result {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    // Convert result to JSON map
    let mut result_map = serde_json::Map::new();
    let columns = result.columns().to_vec();

    // There should be one row of output containing OUT/INOUT values
    // We collect the first row
    let mut found_row = false;
    if let Some(row) = result.flatten().next() {
        found_row = true;
        for (i, col_name) in columns.iter().enumerate() {
            if let Some(val) = row.get_value(i) {
                result_map.insert(col_name.clone(), value_to_json(val));
            } else {
                result_map.insert(col_name.clone(), JsonValue::Null);
            }
        }
    }

    // If there were no OUT params, the result might be empty.
    if !found_row && expected_out.is_empty() {
        if stdout.is_empty() {
            return (
                StatusCode::OK,
                Json(serde_json::json!({ "status": "success" })),
            )
                .into_response();
        } else {
            return (
                StatusCode::OK,
                Json(serde_json::json!({ "status": "success", "result": stdout })),
            )
                .into_response();
        }
    }

    // If there is captured stdout, we can optionally add it to the map
    if !stdout.is_empty() {
        result_map.insert("_stdout".to_string(), JsonValue::String(stdout));
    }

    // Let's just return the result map inside {"result": ...}
    // For single OUT parameter, return {"result": <value>} as per tests
    if result_map.len() == 1 {
        let single_val = result_map.values().next().unwrap().clone();
        (
            StatusCode::OK,
            Json(serde_json::json!({ "result": single_val })),
        )
            .into_response()
    } else {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "result": result_map })),
        )
            .into_response()
    }
}

// Convert serde_json::Value to oxibase::Value
fn json_to_value(json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::null_unknown(),
        JsonValue::Bool(b) => Value::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::null_unknown()
            }
        }
        JsonValue::String(s) => Value::text(s),
        JsonValue::Array(_) | JsonValue::Object(_) => {
            Value::Json(std::sync::Arc::from(json.to_string()))
        }
    }
}

#[derive(Deserialize)]
pub struct SqlRequest {
    pub query: String,
}

pub async fn execute_sql(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<SqlRequest>,
) -> impl IntoResponse {
    let parent_cx = opentelemetry::global::get_text_map_propagator(|prop| {
        prop.extract(&HeaderExtractor(&headers))
    });
    let span = tracing::info_span!("network.request", method = "POST", path = "/api/sql");
    let _ = span.set_parent(parent_cx);
    let _guard = span.enter();

    let sql = payload.query.trim();

    // Check if it's a row-returning query (SELECT, SHOW, EXPLAIN, etc)
    let upper_sql = sql.to_uppercase();
    let is_query = upper_sql.starts_with("SELECT")
        || upper_sql.starts_with("SHOW")
        || upper_sql.starts_with("EXPLAIN")
        || upper_sql.starts_with("DESCRIBE")
        || upper_sql.starts_with("WITH");

    if is_query {
        match state.db.query(sql, ()) {
            Ok(rows_result) => {
                let columns = rows_result.columns().to_vec();
                let mut all_rows = Vec::new();

                for row_res in rows_result {
                    match row_res {
                        Ok(row) => {
                            let mut json_row = serde_json::Map::new();
                            for (i, col_name) in columns.iter().enumerate() {
                                let val =
                                    row.get_value(i).cloned().unwrap_or(Value::null_unknown());
                                json_row.insert(col_name.clone(), value_to_json(&val));
                            }
                            all_rows.push(JsonValue::Object(json_row));
                        }
                        Err(e) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(serde_json::json!({ "error": format!("Row error: {}", e) })),
                            )
                                .into_response();
                        }
                    }
                }

                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "columns": columns,
                        "rows": all_rows
                    })),
                )
                    .into_response()
            }
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Query error: {}", e) })),
            )
                .into_response(),
        }
    } else {
        match state.db.execute(sql, ()) {
            Ok(rows_affected) => (
                StatusCode::OK,
                Json(serde_json::json!({ "rows_affected": rows_affected })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Execution error: {}", e) })),
            )
                .into_response(),
        }
    }
}

pub async fn workspace_execute_sql(
    headers: HeaderMap,
    State(state): State<AppState>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let parent_cx = opentelemetry::global::get_text_map_propagator(|prop| {
        prop.extract(&HeaderExtractor(&headers))
    });
    let span = tracing::info_span!("network.request", method = "POST", path = "/workspace/sql");
    let _ = span.set_parent(parent_cx);
    let _guard = span.enter();

    let sql = match form.get("query") {
        Some(q) => q.trim(),
        None => return (StatusCode::BAD_REQUEST, "Missing query").into_response(),
    };

    let mut context = serde_json::Map::new();
    context.insert("query".to_string(), JsonValue::String(sql.to_string()));

    let upper_sql = sql.to_uppercase();
    let is_query = upper_sql.starts_with("SELECT")
        || upper_sql.starts_with("SHOW")
        || upper_sql.starts_with("EXPLAIN")
        || upper_sql.starts_with("DESCRIBE")
        || upper_sql.starts_with("WITH");

    if is_query {
        match state.db.query(sql, ()) {
            Ok(rows_result) => {
                let columns = rows_result.columns().to_vec();
                let mut all_rows = Vec::new();

                for row_res in rows_result {
                    match row_res {
                        Ok(row) => {
                            let mut json_row = serde_json::Map::new();
                            for (i, col_name) in columns.iter().enumerate() {
                                let val =
                                    row.get_value(i).cloned().unwrap_or(Value::null_unknown());
                                json_row.insert(col_name.clone(), value_to_json(&val));
                            }
                            all_rows.push(JsonValue::Object(json_row));
                        }
                        Err(e) => {
                            context.insert("error".to_string(), JsonValue::String(e.to_string()));
                            break;
                        }
                    }
                }

                context.insert("columns".to_string(), serde_json::json!(columns));
                context.insert("rows".to_string(), JsonValue::Array(all_rows));
            }
            Err(e) => {
                context.insert("error".to_string(), JsonValue::String(e.to_string()));
            }
        }
    } else {
        match state.db.execute(sql, ()) {
            Ok(rows_affected) => {
                context.insert(
                    "rows_affected".to_string(),
                    serde_json::json!(rows_affected),
                );
            }
            Err(e) => {
                context.insert("error".to_string(), JsonValue::String(e.to_string()));
            }
        }
    }

    let env = create_env(state.db.clone());
    let tmpl = match env.get_template("workspace_sql_results.html") {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template load error: {}", e),
            )
                .into_response()
        }
    };

    match tmpl.render(JsonValue::Object(context)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template render error: {}", e),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct WorkspaceTableCreateForm {
    pub table_name: String,
    pub col_name: String,
    pub col_type: String,
}

pub async fn workspace_create_table(
    State(state): State<AppState>,
    Form(form): Form<WorkspaceTableCreateForm>,
) -> impl IntoResponse {
    let sql = format!(
        "CREATE TABLE {} ({} {})",
        form.table_name, form.col_name, form.col_type
    );

    match state.db.execute(&sql, ()) {
        Ok(_) => {
            // Unpoly accepts a redirect or an HTML fragment
            // We'll return an Unpoly-compatible response to close the modal and reload the sidebar
            let html = r#"
            <div class="p-4 bg-green-100 text-green-700 rounded">
                Table created successfully.
                <script>
                    up.emit('table:created');
                    up.layer.dismiss();
                </script>
            </div>
            "#;
            Html(html).into_response()
        }
        Err(e) => {
            let html = format!(
                r#"
            <div class="p-4 bg-red-100 text-red-700 rounded">
                Error creating table: {}
            </div>
            "#,
                e
            );
            (StatusCode::BAD_REQUEST, Html(html)).into_response()
        }
    }
}

pub async fn workspace_get_table_data(
    Path((schema, table)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut context = serde_json::Map::new();
    context.insert("schema".to_string(), JsonValue::String(schema.clone()));
    context.insert("table".to_string(), JsonValue::String(table.clone()));

    let sql = format!("SELECT * FROM {}.{} LIMIT 100", schema, table);

    match state.db.query(&sql, ()) {
        Ok(rows_result) => {
            let columns = rows_result.columns().to_vec();
            let mut all_rows = Vec::new();

            for row_res in rows_result {
                match row_res {
                    Ok(row) => {
                        let mut json_row = serde_json::Map::new();
                        for (i, col_name) in columns.iter().enumerate() {
                            let val = row.get_value(i).cloned().unwrap_or(Value::null_unknown());
                            json_row.insert(col_name.clone(), value_to_json(&val));
                        }
                        all_rows.push(JsonValue::Object(json_row));
                    }
                    Err(e) => {
                        context.insert("error".to_string(), JsonValue::String(e.to_string()));
                        break;
                    }
                }
            }

            context.insert("columns".to_string(), serde_json::json!(columns));
            context.insert("rows".to_string(), JsonValue::Array(all_rows));
        }
        Err(e) => {
            context.insert("error".to_string(), JsonValue::String(e.to_string()));
        }
    }

    let env = create_env(state.db.clone());
    let tmpl = match env.get_template("workspace_data_grid.html") {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template load error: {}", e),
            )
                .into_response()
        }
    };

    match tmpl.render(JsonValue::Object(context)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template render error: {}", e),
        )
            .into_response(),
    }
}

pub async fn workspace_trace_view(
    Path(trace_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut context = serde_json::Map::new();
    context.insert("trace_id".to_string(), JsonValue::String(trace_id.clone()));

    let sql = "SELECT span_id, parent_span_id, name, span_kind, start_time, end_time, duration_ms, status_code, status_message, attributes FROM system.traces WHERE trace_id = ? ORDER BY start_time ASC";

    match state.db.query(sql, vec![Value::text(&trace_id)]) {
        Ok(rows_result) => {
            let columns = rows_result.columns().to_vec();
            let mut all_spans = Vec::new();
            let mut start_time = String::new();
            let mut end_time = String::new();

            for row in rows_result.flatten() {
                let mut json_row = serde_json::Map::new();
                for (i, col_name) in columns.iter().enumerate() {
                    let val = row.get_value(i).cloned().unwrap_or(Value::null_unknown());
                    json_row.insert(col_name.clone(), value_to_json(&val));
                }

                if start_time.is_empty() {
                    if let Some(st) = json_row.get("start_time") {
                        start_time = st.as_str().unwrap_or("").to_string();
                    }
                }
                if let Some(et) = json_row.get("end_time") {
                    end_time = et.as_str().unwrap_or("").to_string();
                }

                all_spans.push(JsonValue::Object(json_row));
            }

            context.insert("spans".to_string(), JsonValue::Array(all_spans.clone()));
            context.insert(
                "spans_json".to_string(),
                JsonValue::String(
                    serde_json::to_string(&all_spans).unwrap_or_else(|_| "[]".to_string()),
                ),
            );
            context.insert(
                "trace_start_time".to_string(),
                JsonValue::String(start_time),
            );
            context.insert("trace_end_time".to_string(), JsonValue::String(end_time));
        }
        Err(e) => {
            context.insert("error".to_string(), JsonValue::String(e.to_string()));
        }
    }

    let env = create_env(state.db.clone());
    let tmpl = match env.get_template("workspace_trace_view.html") {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template load error: {}", e),
            )
                .into_response()
        }
    };

    match tmpl.render(JsonValue::Object(context)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template render error: {}", e),
        )
            .into_response(),
    }
}

pub async fn workspace_run_modal(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut context = serde_json::Map::new();
    let procedure_name = params.get("procedure_name").cloned().unwrap_or_default();
    context.insert(
        "procedure_name".to_string(),
        JsonValue::String(procedure_name.clone()),
    );

    let sql = "SELECT parameters FROM system.procedures WHERE name = ?";
    let mut parameters_json = JsonValue::Array(Vec::new());

    match state
        .db
        .query(sql, vec![Value::text(procedure_name.to_uppercase())])
    {
        Ok(rows_result) => {
            if let Some(row) = rows_result.flatten().next() {
                if let Some(Value::Text(params_str)) = row.get_value(0) {
                    if let Ok(parsed) = serde_json::from_str::<JsonValue>(params_str.as_ref()) {
                        parameters_json = parsed;
                    }
                }
            }
        }
        Err(e) => {
            context.insert("error".to_string(), JsonValue::String(e.to_string()));
        }
    }

    // Filter out OUT parameters from the input form
    if let JsonValue::Array(params_arr) = parameters_json {
        let in_params: Vec<JsonValue> = params_arr
            .into_iter()
            .filter(|p| {
                if let Some(mode) = p.get("mode").and_then(|m| m.as_str()) {
                    mode == "IN" || mode == "INOUT"
                } else {
                    true // default mode is IN
                }
            })
            .collect();
        context.insert("parameters".to_string(), JsonValue::Array(in_params));
    } else {
        context.insert("parameters".to_string(), parameters_json);
    }

    let env = create_env(state.db.clone());
    let tmpl = match env.get_template("workspace_run_modal.html") {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template load error: {}", e),
            )
                .into_response()
        }
    };

    match tmpl.render(JsonValue::Object(context)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template render error: {}", e),
        )
            .into_response(),
    }
}
