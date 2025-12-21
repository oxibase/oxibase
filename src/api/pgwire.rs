// Copyright 2025 Stoolap Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! PostgreSQL wire protocol implementation for Oxibase

use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;
use pgwire::api::query::SimpleQueryHandler;
use pgwire::api::results::{DataRowEncoder, FieldFormat, FieldInfo, QueryResponse, Response};
use pgwire::api::PgWireServerHandlers;
use pgwire::api::Type as FieldType;
use pgwire::error::{ErrorInfo, PgWireResult};

use crate::api::Database;
use crate::core::{types::DataType, Value};

/// Backend factory for creating Oxibase backends
pub struct OxiBaseBackendFactory {
    db: Database,
}

impl OxiBaseBackendFactory {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl PgWireServerHandlers for OxiBaseBackendFactory {
    fn simple_query_handler(&self) -> Arc<impl SimpleQueryHandler> {
        Arc::new(OxiBaseBackend::new(self.db.clone()))
    }
}

/// Oxibase backend implementation
pub struct OxiBaseBackend {
    db: Database,
}

impl OxiBaseBackend {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Convert OxiBase Value to pgwire FieldType
    fn value_to_field_type(value: &Value) -> FieldType {
        match value {
            Value::Null(dt) => match dt {
                DataType::Integer => FieldType::INT8,
                DataType::Float => FieldType::FLOAT8,
                DataType::Text => FieldType::TEXT,
                DataType::Boolean => FieldType::BOOL,
                DataType::Timestamp => FieldType::TIMESTAMPTZ,
                DataType::Json => FieldType::JSON,
                DataType::Null => FieldType::UNKNOWN,
            },
            Value::Integer(_) => FieldType::INT8,
            Value::Float(_) => FieldType::FLOAT8,
            Value::Text(_) => FieldType::TEXT,
            Value::Boolean(_) => FieldType::BOOL,
            Value::Timestamp(_) => FieldType::TIMESTAMPTZ,
            Value::Json(_) => FieldType::JSON,
        }
    }

    /// Encode a Value for pgwire DataRow
    fn encode_value(value: &Value, encoder: &mut DataRowEncoder) -> PgWireResult<()> {
        match value {
            Value::Null(_) => encoder.encode_field(&None::<i32>)?,
            Value::Integer(v) => encoder.encode_field(v)?,
            Value::Float(v) => encoder.encode_field(v)?,
            Value::Text(s) => encoder.encode_field(&s.as_ref())?,
            Value::Boolean(b) => encoder.encode_field(b)?,
            Value::Timestamp(t) => encoder.encode_field(t)?,
            Value::Json(s) => encoder.encode_field(&s.as_ref())?,
        }
        Ok(())
    }

    /// Handle SELECT queries
    fn handle_select_query(&self, query: &str) -> PgWireResult<Vec<Response>> {
        match self.db.query(query, ()) {
            Ok(rows) => {
                let all_rows: Vec<Result<super::rows::ResultRow, crate::core::Error>> =
                    rows.collect();
                let columns: Arc<Vec<String>> = Arc::new(
                    all_rows
                        .first()
                        .and_then(|r| r.as_ref().ok())
                        .map(|row| row.columns().to_vec())
                        .unwrap_or_default(),
                );
                let mut field_infos = Vec::new();

                // Use first row to infer field types, or default to TEXT
                let first_row = all_rows.first().and_then(|r| r.as_ref().ok());

                 for (i, col_name) in columns.iter().enumerate() {
                     let field_type = if let Some(row) = first_row {
                        if let Some(value) = row.get_value(i) {
                            Self::value_to_field_type(value)
                        } else {
                            FieldType::TEXT
                        }
                    } else {
                        FieldType::TEXT
                    };

                    field_infos.push(FieldInfo::new(
                        col_name.clone(),
                        None,
                        None,
                        field_type,
                        FieldFormat::Text,
                    ));
                }

                let mut data_rows = Vec::new();

                for row_result in all_rows {
                    match row_result {
                        Ok(row) => {
                            let mut encoder =
                                DataRowEncoder::new(std::sync::Arc::new(field_infos.clone()));
                            for i in 0..columns.len() {
                                if let Some(value) = row.get_value(i) {
                                    Self::encode_value(value, &mut encoder)?;
                                } else {
                                    encoder.encode_field(&None::<i32>)?;
                                }
                            }
                            data_rows.push(encoder.take_row());
                        }
                        Err(e) => {
                            return Ok(vec![Response::Error(Box::new(ErrorInfo::new(
                                "ERROR".to_owned(),
                                "XX000".to_owned(),
                                format!("Error fetching row: {}", e),
                            )))]);
                        }
                    }
                }

                Ok(vec![Response::Query(QueryResponse::new(
                    std::sync::Arc::new(field_infos),
                    stream::iter(data_rows.into_iter().map(Ok)),
                ))])
            }
            Err(e) => Ok(vec![Response::Error(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "XX000".to_owned(),
                format!("Query execution failed: {}", e),
            )))]),
        }
    }

    /// Handle DML queries (INSERT, UPDATE, DELETE)
    fn handle_dml_query(&self, query: &str) -> PgWireResult<Vec<Response>> {
        match self.db.execute(query, ()) {
            Ok(rows_affected) => {
                // Determine the tag based on query type
                let tag = if query.trim().to_uppercase().starts_with("INSERT") {
                    "INSERT"
                } else if query.trim().to_uppercase().starts_with("UPDATE") {
                    "UPDATE"
                } else if query.trim().to_uppercase().starts_with("DELETE") {
                    "DELETE"
                } else {
                    "COMMAND"
                };

                // For now, return error since we don't know the correct Response variant
                Ok(vec![Response::Error(Box::new(ErrorInfo::new(
                    "INFO".to_owned(),
                    "00000".to_owned(),
                    format!("{} {}", tag, rows_affected),
                )))])
            }
            Err(e) => Ok(vec![Response::Error(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "XX000".to_owned(),
                format!("Query execution failed: {}", e),
            )))]),
        }
    }
}

#[async_trait]
impl SimpleQueryHandler for OxiBaseBackend {
    async fn do_query<C>(&self, _client: &mut C, query: &str) -> PgWireResult<Vec<Response>>
    where
        C: pgwire::api::ClientInfo + Unpin + Send + Sync,
    {
        let trimmed_query = query.trim().to_uppercase();

        if trimmed_query.starts_with("SELECT") {
            self.handle_select_query(query)
        } else {
            self.handle_dml_query(query)
        }
    }
}
