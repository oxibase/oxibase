// This file handles the logic for the Axum routes.
// We import required Axum and Oxibase types to be used across handlers.

use crate::api::Database;
use crate::server::AppState;
use crate::Value;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
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
