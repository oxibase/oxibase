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
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

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
fn table_exists(db: &Database, table_name: &str) -> Result<bool, String> {
    let query = "SELECT 1 FROM information_schema.tables WHERE table_name = ?";
    let rows = db
        .query(query, vec![Value::text(table_name)])
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
    State(state): State<AppState>,
) -> impl IntoResponse {
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
        "SELECT template_name, context_query FROM routes.definitions WHERE method = ? AND path = ?";
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
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Map<String, JsonValue>>,
) -> impl IntoResponse {
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
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Map<String, JsonValue>>,
) -> impl IntoResponse {
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
    State(state): State<AppState>,
) -> impl IntoResponse {
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

    // Execute
    let result =
        crate::functions::context::with_http_headers(headers, || state.db.query(&sql, args));

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
        return (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success" })),
        )
            .into_response();
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
