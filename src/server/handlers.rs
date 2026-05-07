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
    // If the table name contains a dot, we need to check schema AND table name
    let query = if table_name.contains('.') {
        let parts: Vec<&str> = table_name.splitn(2, '.').collect();
        "SELECT 1 FROM information_schema.tables WHERE table_schema = ? AND table_name = ?"
    } else {
        "SELECT 1 FROM information_schema.tables WHERE table_name = ?"
    };

    let params = if table_name.contains('.') {
        let parts: Vec<&str> = table_name.splitn(2, '.').collect();
        vec![Value::text(parts[0]), Value::text(parts[1])]
    } else {
        vec![Value::text(table_name)]
    };

    let rows = db
        .query(query, params)
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
