// Copyright 2025 Oxibase Contributors
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

//! System Schema virtual tables implementation
//!
//! This module provides virtual table implementations for system.* tables:
//! - system.tables
//! - system.columns
//! - system.transactions

use std::sync::Arc;

use crate::core::{DataType, Error, Result, Row, Value};
use crate::parser::ast::*;
use crate::storage::traits::QueryResult;

use super::context::ExecutionContext;
use super::result::ExecutorMemoryResult;
use super::Executor;

impl Executor {
    /// Execute queries against system virtual tables
    pub(crate) fn execute_system_schema_table(
        &self,
        schema_table: &str,
        _stmt: &SelectStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        match schema_table {
            "tables" => self.build_system_tables_result(),
            "columns" => self.build_system_columns_result(),
            "transactions" => self.build_system_transactions_result(),
            _ => Err(Error::TableNotFoundByName(format!(
                "system.{}",
                schema_table
            ))),
        }
    }

    /// Build system.tables result
    fn build_system_tables_result(&self) -> Result<Box<dyn QueryResult>> {
        // Read directly from the in-memory engine catalog
        let schemas_guard = self.engine.schemas.read().unwrap();

        let columns = vec![
            "schema_name".to_string(),
            "table_name".to_string(),
            "created_at".to_string(),
            "updated_at".to_string(),
        ];

        let mut rows: Vec<Row> = Vec::new();

        for (schema_name, table_map) in schemas_guard.iter() {
            for (table_name, schema) in table_map.iter() {
                rows.push(Row::from_values(vec![
                    Value::Text(Arc::from(schema_name.as_str())),
                    Value::Text(Arc::from(table_name.as_str())),
                    Value::Timestamp(schema.created_at),
                    Value::Timestamp(schema.updated_at),
                ]));
            }
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Build system.columns result
    fn build_system_columns_result(&self) -> Result<Box<dyn QueryResult>> {
        let schemas_guard = self.engine.schemas.read().unwrap();

        let columns = vec![
            "schema_name".to_string(),
            "table_name".to_string(),
            "column_name".to_string(),
            "ordinal_position".to_string(),
            "data_type".to_string(),
            "is_nullable".to_string(),
            "is_primary_key".to_string(),
            "column_default".to_string(),
        ];

        let mut rows: Vec<Row> = Vec::new();

        for (schema_name, table_map) in schemas_guard.iter() {
            for (table_name, schema) in table_map.iter() {
                for (pos, col) in schema.columns.iter().enumerate() {
                    let ordinal = (pos + 1) as i64;
                    let data_type = format!("{:?}", col.data_type);

                    // Get default value expression
                    let default_value = col.default_expr.as_ref().and_then(|expr| {
                        let expr_str = expr.to_string();
                        let trimmed = expr_str.trim();
                        if trimmed.is_empty() || trimmed == "NULL" {
                            None
                        } else {
                            Some(trimmed.to_string())
                        }
                    });

                    rows.push(Row::from_values(vec![
                        Value::Text(Arc::from(schema_name.as_str())),
                        Value::Text(Arc::from(table_name.as_str())),
                        Value::Text(Arc::from(col.name.as_str())),
                        Value::Integer(ordinal),
                        Value::Text(Arc::from(data_type.as_str())),
                        Value::Boolean(col.nullable),
                        Value::Boolean(col.primary_key),
                        match default_value {
                            Some(s) => Value::Text(Arc::from(s.as_str())),
                            None => Value::Null(DataType::Text),
                        },
                    ]));
                }
            }
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Build system.transactions result
    fn build_system_transactions_result(&self) -> Result<Box<dyn QueryResult>> {
        let registry = self.engine.registry();

        let columns = vec![
            "id".to_string(),
            "state".to_string(),
            "started_at".to_string(),
        ];

        let mut rows: Vec<Row> = Vec::new();

        // Note: The registry doesn't currently expose a direct list of active transaction states
        // This is a minimal placeholder until the registry supports listing active transactions directly
        // For debugging, we could iterate up to next_txn_id and check status, but that's expensive
        let active_count = registry.active_count();
        if active_count > 0 {
            // Just returning a summary row for now
            rows.push(Row::from_values(vec![
                Value::Integer(0), // Placeholder ID
                Value::Text(Arc::from(format!("ACTIVE ({})", active_count).as_str())),
                Value::Null(DataType::Timestamp),
            ]));
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }
}
