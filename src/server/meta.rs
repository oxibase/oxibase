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

use crate::server::handlers::value_to_json;
use crate::server::AppState;
use crate::Value;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub is_nullable: Option<bool>,
}

#[derive(Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub schema: Option<String>,
    pub columns: Vec<ColumnDefinition>,
}

pub async fn create_table(
    State(state): State<AppState>,
    Json(payload): Json<TableDefinition>,
) -> impl IntoResponse {
    let schema = payload.schema.unwrap_or_else(|| "public".to_string());

    if payload.columns.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Table must have at least one column" })),
        )
            .into_response();
    }

    let mut col_defs = Vec::new();
    for col in payload.columns {
        let nullable = if col.is_nullable.unwrap_or(true) {
            ""
        } else {
            " NOT NULL"
        };
        col_defs.push(format!("{} {}{}", col.name, col.data_type, nullable));
    }

    let sql = format!(
        "CREATE TABLE {}.{} ({})",
        schema,
        payload.name,
        col_defs.join(", ")
    );

    match state.db.execute(&sql, ()) {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({ "status": "success", "table": format!("{}.{}", schema, payload.name) }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn drop_table(
    Path(table_path): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let (schema, name) = if let Some(dot_pos) = table_path.find('.') {
        (&table_path[..dot_pos], &table_path[dot_pos + 1..])
    } else {
        ("public", table_path.as_str())
    };

    let sql = format!("DROP TABLE {}.{}", schema, name);

    match state.db.execute(&sql, ()) {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct AddColumnPayload {
    pub table_id: String,
    pub name: String,
    pub data_type: String,
    pub is_nullable: Option<bool>,
}

pub async fn add_column(
    State(state): State<AppState>,
    Json(payload): Json<AddColumnPayload>,
) -> impl IntoResponse {
    let (schema, table_name) = if let Some(dot_pos) = payload.table_id.find('.') {
        (
            &payload.table_id[..dot_pos],
            &payload.table_id[dot_pos + 1..],
        )
    } else {
        ("public", payload.table_id.as_str())
    };

    let nullable = if payload.is_nullable.unwrap_or(true) {
        "NULL"
    } else {
        "NOT NULL"
    };
    let sql = format!(
        "ALTER TABLE {}.{} ADD COLUMN {} {} {}",
        schema, table_name, payload.name, payload.data_type, nullable
    );

    match state.db.execute(&sql, ()) {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
use serde_json::Value as JsonValue;

/// Helper function to execute a query and return JSON rows
async fn execute_query_as_json(
    state: &AppState,
    query: &str,
    args: Vec<Value>,
) -> axum::response::Response {
    match state.db.query(query, args) {
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
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": format!("Row error: {}", e) })),
                        )
                            .into_response();
                    }
                }
            }

            (StatusCode::OK, Json(JsonValue::Array(all_rows))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Query error: {}", e) })),
        )
            .into_response(),
    }
}

pub async fn list_schemas(State(state): State<AppState>) -> axum::response::Response {
    let query = "SELECT DISTINCT table_schema as schema_name FROM information_schema.tables WHERE table_schema IS NOT NULL ORDER BY table_schema";
    execute_query_as_json(&state, query, vec![]).await
}

pub async fn list_tables(State(state): State<AppState>) -> axum::response::Response {
    let query = "SELECT table_catalog, table_schema, table_name, table_type FROM information_schema.tables ORDER BY table_schema, table_name";
    execute_query_as_json(&state, query, vec![]).await
}

pub async fn list_views(State(state): State<AppState>) -> axum::response::Response {
    let query = "SELECT table_catalog, table_schema, table_name, view_definition FROM information_schema.views ORDER BY table_schema, table_name";
    execute_query_as_json(&state, query, vec![]).await
}

pub async fn list_columns(State(state): State<AppState>) -> axum::response::Response {
    // In a real implementation we might want to filter by table, but for now we list all or allow query params
    // Let's keep it simple and just list all for the schema explorer to process, or we could add query params later.
    let query = "SELECT table_schema, table_name, column_name, data_type, is_nullable, column_default FROM information_schema.columns ORDER BY table_schema, table_name, ordinal_position";
    execute_query_as_json(&state, query, vec![]).await
}

pub async fn list_functions(State(state): State<AppState>) -> axum::response::Response {
    let query = "SELECT function_schema, function_name, function_type, data_type, is_deterministic FROM information_schema.functions ORDER BY function_schema, function_name";
    execute_query_as_json(&state, query, vec![]).await
}

pub async fn list_indexes(State(state): State<AppState>) -> axum::response::Response {
    let query = "SELECT table_schema, table_name, index_name, column_name, index_type, non_unique FROM information_schema.statistics ORDER BY table_schema, table_name, index_name, seq_in_index";
    execute_query_as_json(&state, query, vec![]).await
}

pub async fn list_constraints(State(state): State<AppState>) -> axum::response::Response {
    // Basic implementation since information_schema.table_constraints might not exist yet
    // We'll return an empty array if the query fails, or implement it if possible
    let query = "SELECT 'public' as constraint_schema, 'unknown' as constraint_name, 'unknown' as table_name, 'UNKNOWN' as constraint_type WHERE 1=0";
    execute_query_as_json(&state, query, vec![]).await
}

pub async fn list_triggers(State(state): State<AppState>) -> axum::response::Response {
    // Basic implementation for triggers
    let query = "SELECT schema_name as trigger_schema, trigger_name, table_name, event, timing FROM system.triggers ORDER BY schema_name, trigger_name";
    // We'll try to execute it, if it fails because system.triggers doesn't exist, we'll return an empty array manually.
    match state.db.query(query, vec![]) {
        Ok(_) => execute_query_as_json(&state, query, vec![]).await,
        Err(_) => (StatusCode::OK, Json(JsonValue::Array(vec![]))).into_response(),
    }
}
