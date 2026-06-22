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

use crate::api::{Database, NamedParams};
use crate::server::template::create_env;
use crate::server::AppState;
use crate::Value;
use axum::{
    extract::{Path, Query, Request, State},
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
        Value::Timestamp(ts) => serde_json::json!(ts.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
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

fn url_decode(val: &str) -> String {
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
    decoded
}

fn match_and_extract_path_vars(
    route_pattern: &str,
    request_path: &str,
) -> Option<std::collections::HashMap<String, String>> {
    let route_parts: Vec<&str> = route_pattern.split('/').filter(|s| !s.is_empty()).collect();
    let request_parts: Vec<&str> = request_path.split('/').filter(|s| !s.is_empty()).collect();
    if route_parts.len() != request_parts.len() {
        return None;
    }
    let mut params = std::collections::HashMap::new();
    for (r_part, req_part) in route_parts.iter().zip(request_parts.iter()) {
        if r_part.starts_with('{') && r_part.ends_with('}') {
            let var_name = &r_part[1..r_part.len() - 1];
            params.insert(var_name.to_string(), req_part.to_string());
        } else if let Some(var_name) = r_part.strip_prefix(':') {
            params.insert(var_name.to_string(), req_part.to_string());
        } else if r_part != req_part {
            return None;
        }
    }
    Some(params)
}

/// Fallback route handler for dynamic database-driven templates
pub async fn dynamic_route_handler(
    State(state): State<AppState>,
    req: Request,
) -> impl IntoResponse {
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();
    let query_str = req.uri().query().map(|s| s.to_string());

    // Extract body bytes
    let (_parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => axum::body::Bytes::new(),
    };

    // Lookup all routes for this method
    let query = "SELECT template_name, context_query, path FROM interface.routes WHERE method = ?";
    let rows_res = match state.db.query(query, vec![Value::text(&method)]) {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error finding routes: {}", e),
            )
                .into_response()
        }
    };

    let mut matched_route = None;
    for row_res in rows_res {
        let row = match row_res {
            Ok(r) => r,
            Err(_) => continue,
        };
        let t_name = row
            .get_value(0)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let ctx_q = row
            .get_value(1)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let r_path = row
            .get_value(2)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if let Some(extracted) = match_and_extract_path_vars(&r_path, &path) {
            matched_route = Some((t_name, ctx_q, extracted));
            break;
        }
    }

    let (template_name, context_query, path_params) = match matched_route {
        Some(val) => val,
        None => return (StatusCode::NOT_FOUND, "Not Found").into_response(),
    };

    // Coalesce parameters
    let mut context_params = std::collections::HashMap::<String, Value>::new();
    for (k, v) in path_params {
        context_params.insert(k, Value::text(v));
    }

    if let Some(qs) = query_str {
        for pair in qs.split('&') {
            if let Some((key, val)) = pair.split_once('=') {
                let decoded = url_decode(val);
                context_params.insert(key.to_string(), Value::text(decoded));
            }
        }
    }

    if !body_bytes.is_empty() {
        if let Ok(serde_json::Value::Object(map)) =
            serde_json::from_slice::<serde_json::Value>(&body_bytes)
        {
            for (key, val) in map {
                let oxibase_val = match val {
                    serde_json::Value::Null => Value::null_unknown(),
                    serde_json::Value::Bool(b) => Value::boolean(b),
                    serde_json::Value::Number(num) => {
                        if let Some(i) = num.as_i64() {
                            Value::integer(i)
                        } else if let Some(f) = num.as_f64() {
                            Value::float(f)
                        } else {
                            Value::null_unknown()
                        }
                    }
                    serde_json::Value::String(s) => Value::text(s),
                    _ => Value::text(val.to_string()),
                };
                context_params.insert(key, oxibase_val);
            }
        } else {
            // Try to parse as urlencoded form data
            if let Ok(body_str) = String::from_utf8(body_bytes.to_vec()) {
                for pair in body_str.split('&') {
                    if let Some((key, val)) = pair.split_once('=') {
                        let decoded_key = url_decode(key);
                        let decoded_val = url_decode(val);
                        context_params.insert(decoded_key, Value::text(decoded_val));
                    }
                }
            }
        }
    }

    // Build context
    let mut template_ctx = serde_json::Map::new();

    // Map context_params into template context "params" and root context for compatibility
    let mut params_json_map = serde_json::Map::new();
    for (k, v) in &context_params {
        let json_val = value_to_json(v);
        params_json_map.insert(k.clone(), json_val.clone());
        template_ctx.insert(k.clone(), json_val);
    }
    template_ctx.insert(
        "params".to_string(),
        serde_json::Value::Object(params_json_map),
    );

    if let Some(ctx_query) = context_query {
        // Run the query to fetch dynamic context
        let mut compiled_query = ctx_query.clone();

        // Special case: SQL editor query execution
        if path == "/workspace/sql" {
            if let Some(q_val) = context_params.get("query").and_then(|v| v.as_str()) {
                compiled_query = q_val.to_string();
            }
        }

        if compiled_query.contains(":schema") || compiled_query.contains(":table") {
            let s_val = context_params
                .get("schema")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let t_val = context_params
                .get("table")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if s_val.chars().all(|c| c.is_alphanumeric() || c == '_')
                && t_val.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                compiled_query = compiled_query
                    .replace(":schema", s_val)
                    .replace(":table", t_val);
            }
        }

        let is_query = {
            let u = compiled_query.to_uppercase();
            u.starts_with("SELECT")
                || u.starts_with("SHOW")
                || u.starts_with("EXPLAIN")
                || u.starts_with("DESCRIBE")
                || u.starts_with("WITH")
        };

        if is_query {
            let named = NamedParams::from(context_params.clone());
            let ctx_rows_res = match state.db.query_named(&compiled_query, named) {
                Ok(res) => res,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Query error: {}", e),
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

            template_ctx.insert("data".to_string(), JsonValue::Array(all_rows));
            template_ctx.insert("columns".to_string(), serde_json::json!(cols));
        } else {
            let named = NamedParams::from(context_params.clone());
            match state.db.execute_named(&compiled_query, named) {
                Ok(rows_affected) => {
                    template_ctx.insert(
                        "rows_affected".to_string(),
                        serde_json::json!(rows_affected),
                    );
                    template_ctx.insert("data".to_string(), JsonValue::Array(vec![]));
                    template_ctx.insert(
                        "columns".to_string(),
                        serde_json::json!(Vec::<String>::new()),
                    );
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Execution error: {}", e),
                    )
                        .into_response()
                }
            }
        }
    } else {
        template_ctx.insert("data".to_string(), JsonValue::Array(vec![]));
        template_ctx.insert(
            "columns".to_string(),
            serde_json::json!(Vec::<String>::new()),
        );
    }

    // Copy data to template-specific variable aliases for backwards compatibility
    let data_val = template_ctx
        .get("data")
        .cloned()
        .unwrap_or(JsonValue::Array(vec![]));
    template_ctx.insert("spans".to_string(), data_val.clone());
    template_ctx.insert("logs".to_string(), data_val.clone());
    template_ctx.insert("traces".to_string(), data_val.clone());
    template_ctx.insert("rows".to_string(), data_val.clone());

    // Inject trace ID
    if let Some(tid) = context_params.get("trace_id") {
        template_ctx.insert("trace_id".to_string(), value_to_json(tid));
    }

    // Populate trace start/end time
    let mut trace_start_time = String::new();
    let mut trace_end_time = String::new();
    if let JsonValue::Array(ref spans) = data_val {
        for s in spans {
            if let Some(st) = s.get("start_time").and_then(|v| v.as_str()) {
                if trace_start_time.is_empty() || st < trace_start_time.as_str() {
                    trace_start_time = st.to_string();
                }
            }
            if let Some(et) = s.get("end_time").and_then(|v| v.as_str()) {
                if trace_end_time.is_empty() || et > trace_end_time.as_str() {
                    trace_end_time = et.to_string();
                }
            }
        }
    }
    template_ctx.insert(
        "trace_start_time".to_string(),
        JsonValue::String(trace_start_time),
    );
    template_ctx.insert(
        "trace_end_time".to_string(),
        JsonValue::String(trace_end_time),
    );
    template_ctx.insert(
        "spans_json".to_string(),
        JsonValue::String(serde_json::to_string(&data_val).unwrap_or_else(|_| "[]".to_string())),
    );

    // Inject log histogram and filters for workspace observe logs
    if path == "/workspace/observe/logs" {
        let mut histogram = serde_json::Map::new();
        histogram.insert("ERROR".to_string(), serde_json::json!(0));
        histogram.insert("WARN".to_string(), serde_json::json!(0));
        histogram.insert("INFO".to_string(), serde_json::json!(0));
        histogram.insert("DEBUG".to_string(), serde_json::json!(0));

        if let Ok(res) = state
            .db
            .query("SELECT level, COUNT(*) FROM system.logs GROUP BY level", ())
        {
            for row in res.flatten() {
                let lvl = row
                    .get_value(0)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let count = row.get_value(1).and_then(|v| v.as_int64()).unwrap_or(0);
                histogram.insert(lvl.to_string(), serde_json::json!(count));
            }
        }
        template_ctx.insert("histogram".to_string(), JsonValue::Object(histogram));

        // Get filters from params
        let level = context_params
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("all")
            .to_string();
        let search = context_params
            .get("search")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let limit_str = context_params
            .get("limit")
            .and_then(|v| v.as_str())
            .unwrap_or("100")
            .to_string();
        let offset_str = context_params
            .get("offset")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();
        let auto_refresh = context_params
            .get("auto_refresh")
            .and_then(|v| v.as_str())
            .unwrap_or("false")
            .to_string();

        let limit: i32 = limit_str.parse().unwrap_or(100);
        let offset: i32 = offset_str.parse().unwrap_or(0);

        let mut filter_map = serde_json::Map::new();
        filter_map.insert("level".to_string(), JsonValue::String(level));
        filter_map.insert("search".to_string(), JsonValue::String(search));
        filter_map.insert("limit".to_string(), serde_json::json!(limit));
        template_ctx.insert("filters".to_string(), JsonValue::Object(filter_map));
        template_ctx.insert("auto_refresh".to_string(), JsonValue::String(auto_refresh));
        template_ctx.insert("has_more".to_string(), JsonValue::Bool(false)); // can default to false or calculate
        template_ctx.insert("next_offset".to_string(), serde_json::json!(offset + limit));
    }

    // Inject trace filters and parameters for workspace observe traces
    if path == "/workspace/observe/traces" {
        let search = context_params
            .get("search")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let status = context_params
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("all")
            .to_string();
        let limit_str = context_params
            .get("limit")
            .and_then(|v| v.as_str())
            .unwrap_or("100")
            .to_string();
        let auto_refresh = context_params
            .get("auto_refresh")
            .and_then(|v| v.as_str())
            .unwrap_or("false")
            .to_string();

        let limit: i32 = limit_str.parse().unwrap_or(100);

        let mut filter_map = serde_json::Map::new();
        filter_map.insert("search".to_string(), JsonValue::String(search));
        filter_map.insert("status".to_string(), JsonValue::String(status));
        filter_map.insert("limit".to_string(), serde_json::json!(limit));
        template_ctx.insert("filters".to_string(), JsonValue::Object(filter_map));
        template_ctx.insert("auto_refresh".to_string(), JsonValue::String(auto_refresh));
    }

    // Inject parameters for run_modal
    if path == "/workspace/run_modal" {
        let mut parameters_json = JsonValue::Array(Vec::new());
        if let JsonValue::Array(ref rows) = data_val {
            if !rows.is_empty() {
                if let Some(JsonValue::String(ref params_str)) = rows[0].get("parameters") {
                    if let Ok(parsed) = serde_json::from_str::<JsonValue>(params_str) {
                        parameters_json = parsed;
                    }
                }
            }
        }

        if let JsonValue::Array(params_arr) = parameters_json {
            let in_params: Vec<JsonValue> = params_arr
                .into_iter()
                .filter(|p| {
                    if let Some(mode) = p.get("mode").and_then(|m| m.as_str()) {
                        mode == "IN" || mode == "INOUT"
                    } else {
                        true
                    }
                })
                .collect();
            template_ctx.insert("parameters".to_string(), JsonValue::Array(in_params));
        } else {
            template_ctx.insert("parameters".to_string(), parameters_json);
        }
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

    match tmpl.render(JsonValue::Object(template_ctx)) {
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

    let mut is_function = false;

    let procedure = match registry.get_procedure(&procedure_name_upper) {
        Some(proc) => proc,
        None => {
            if let Some(info) = registry.get_info(&procedure_name_upper) {
                is_function = true;
                let query = "SELECT language FROM system.functions WHERE UPPER(name) = ?";
                let mut lang = "rhai".to_string();
                if let Ok(rows) = state
                    .db
                    .query(query, vec![crate::Value::text(&procedure_name_upper)])
                {
                    for r in rows.flatten() {
                        if let Some(crate::Value::Text(s)) = r.get_value(0) {
                            lang = s.to_string();
                        }
                    }
                }

                // Get parameter names if stored, or default them
                let mut param_names = Vec::new();
                let query_params = "SELECT parameters FROM system.functions WHERE UPPER(name) = ?";
                if let Ok(rows) = state.db.query(
                    query_params,
                    vec![crate::Value::text(&procedure_name_upper)],
                ) {
                    for r in rows.flatten() {
                        if let Some(crate::Value::Text(s)) = r.get_value(0) {
                            if let Ok(serde_json::Value::Array(arr)) =
                                serde_json::from_str::<serde_json::Value>(s.as_ref())
                            {
                                for p in arr {
                                    if let Some(name) = p.get("name").and_then(|v| v.as_str()) {
                                        param_names.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                crate::storage::procedures::StoredProcedure {
                    id: 0,
                    schema: Some("PUBLIC".to_string()),
                    name: procedure_name_upper.clone(),
                    parameters: info
                        .signature
                        .argument_types
                        .iter()
                        .enumerate()
                        .map(|(i, dtype)| {
                            let name = if i < param_names.len() {
                                param_names[i].clone()
                            } else {
                                format!("arg{}", i)
                            };
                            let type_str = match dtype {
                                crate::functions::FunctionDataType::Any => "ANY",
                                crate::functions::FunctionDataType::Integer => "INTEGER",
                                crate::functions::FunctionDataType::Float => "FLOAT",
                                crate::functions::FunctionDataType::String => "TEXT",
                                crate::functions::FunctionDataType::Boolean => "BOOLEAN",
                                crate::functions::FunctionDataType::Timestamp => "TIMESTAMP",
                                crate::functions::FunctionDataType::Date => "DATE",
                                crate::functions::FunctionDataType::Time => "TIME",
                                crate::functions::FunctionDataType::DateTime => "DATETIME",
                                crate::functions::FunctionDataType::Json => "JSON",
                                crate::functions::FunctionDataType::Unknown => "UNKNOWN",
                            };
                            crate::storage::procedures::StoredProcedureParameter {
                                mode: "IN".to_string(),
                                name,
                                data_type: type_str.to_string(),
                            }
                        })
                        .collect(),
                    language: lang,
                    code: String::new(),
                }
            } else {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": format!("Routine '{}' not found", procedure_name) })),
                ).into_response();
            }
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

    // Now, we can formulate a CALL or SELECT statement and execute it with parameters
    let placeholders: Vec<String> = (1..=args.len()).map(|i| format!("${}", i)).collect();
    let sql = if is_function {
        format!("SELECT {}({})", procedure_name, placeholders.join(", "))
    } else {
        format!("CALL {}({})", procedure_name, placeholders.join(", "))
    };

    let debug_controller = std::sync::Arc::clone(&state.debug_controller);
    let db_clone = std::sync::Arc::clone(&state.db);
    let (result, stdout) = tokio::task::spawn_blocking(move || {
        crate::functions::context::with_http_headers_and_debug(
            headers,
            Some(debug_controller),
            || {
                crate::functions::context::set_current_procedure_name(Some(procedure_name_upper));
                let res = db_clone.query(&sql, args);
                crate::functions::context::set_current_procedure_name(None);
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
