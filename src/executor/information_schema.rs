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

//! Information Schema virtual tables implementation
//!
//! This module provides virtual table implementations for information_schema.* tables:
//! - information_schema.tables
//! - information_schema.columns
//! - information_schema.functions
//! - information_schema.views
//! - information_schema.statistics
//! - information_schema.sequences

use std::sync::Arc;

use crate::core::{DataType, Error, Result, Row, Value};
use crate::parser::{ast::*, Parser};
use crate::storage::traits::{Engine, QueryResult};

use super::context::ExecutionContext;
use super::result::ExecutorMemoryResult;
use super::Executor;

impl Executor {
    /// Execute queries against information_schema virtual tables
    pub(crate) fn execute_information_schema_table(
        &self,
        schema_table: &str,
        _stmt: &SelectStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        match schema_table {
            "tables" => self.build_tables_result(),
            "columns" => self.build_columns_result(),
            "functions" => self.build_functions_result(),
            "views" => self.build_views_result(),
            "statistics" => self.build_statistics_result(),
            "sequences" => self.build_sequences_result(),
            _ => Err(Error::TableNotFoundByName(format!(
                "information_schema.{}",
                schema_table
            ))),
        }
    }

    /// Build information_schema.tables result
    fn build_tables_result(&self) -> Result<Box<dyn QueryResult>> {
        let tx = self.engine.begin_transaction()?;

        // Get all tables
        let table_names = tx.list_tables()?;
        let view_names = self.engine.list_views()?;

        // Columns: table_catalog, table_schema, table_name, table_type
        let columns = vec![
            "table_catalog".to_string(),
            "table_schema".to_string(),
            "table_name".to_string(),
            "table_type".to_string(),
        ];

        let mut rows: Vec<Row> = Vec::new();

        // Add tables
        for table_name in table_names {
            // Parse qualified table names: if it contains a dot, split into schema and name
            let (schema_name, actual_table_name) = if let Some(dot_pos) = table_name.find('.') {
                let schema = &table_name[..dot_pos];
                let name = &table_name[dot_pos + 1..];
                (schema.to_string(), name.to_string())
            } else {
                ("public".to_string(), table_name.clone())
            };

            rows.push(Row::from_values(vec![
                Value::Text(Arc::from("def")),       // catalog
                Value::Text(Arc::from(schema_name)), // schema
                Value::Text(Arc::from(actual_table_name)),
                Value::Text(Arc::from("BASE TABLE")),
            ]));
        }

        // Add views (views are global, no schema)
        for view_name in view_names {
            rows.push(Row::from_values(vec![
                Value::Text(Arc::from("def")),            // catalog
                Value::Null(crate::core::DataType::Text), // schema (NULL for views)
                Value::Text(Arc::from(view_name.as_str())),
                Value::Text(Arc::from("VIEW")),
            ]));
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Build information_schema.columns result
    fn build_columns_result(&self) -> Result<Box<dyn QueryResult>> {
        let tx = self.engine.begin_transaction()?;
        let table_names = tx.list_tables()?;

        // Columns: table_catalog, table_schema, table_name, column_name, ordinal_position,
        //          column_default, is_nullable, data_type, character_maximum_length,
        //          numeric_precision, numeric_scale
        let columns = vec![
            "table_catalog".to_string(),
            "table_schema".to_string(),
            "table_name".to_string(),
            "column_name".to_string(),
            "ordinal_position".to_string(),
            "column_default".to_string(),
            "is_nullable".to_string(),
            "data_type".to_string(),
            "character_maximum_length".to_string(),
            "numeric_precision".to_string(),
            "numeric_scale".to_string(),
        ];

        let mut rows: Vec<Row> = Vec::new();

        for table_name in table_names {
            let table = tx.get_table(&table_name)?;
            let table_schema = table.schema();

            // Parse qualified table names
            let (schema_name, actual_table_name) = if let Some(dot_pos) = table_name.find('.') {
                let schema = &table_name[..dot_pos];
                let name = &table_name[dot_pos + 1..];
                (schema.to_string(), name.to_string())
            } else {
                ("public".to_string(), table_name.clone())
            };

            for (pos, col) in table_schema.columns.iter().enumerate() {
                let ordinal = (pos + 1) as i64;

                // Get default value
                let default_value = col.default_expr.as_ref().and_then(|expr| {
                    let expr_str = expr.to_string();
                    let trimmed = expr_str.trim();
                    // Skip empty strings (after trim), and "NULL" literals
                    if trimmed.is_empty() || trimmed == "NULL" {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                });

                // Get nullable
                let is_nullable = if col.nullable { "YES" } else { "NO" };

                // Determine type string
                let data_type = format!("{:?}", col.data_type);

                // Get character maximum length (for TEXT types)
                let char_max_len = match col.data_type {
                    crate::core::types::DataType::Text => Some(Value::Integer(65535)), // Default TEXT length
                    _ => None,
                };

                // Get numeric precision/scale (for INTEGER/FLOAT)
                let (num_precision, num_scale) = match col.data_type {
                    crate::core::types::DataType::Integer => {
                        (Some(Value::Integer(64)), Some(Value::Integer(0)))
                    }
                    crate::core::types::DataType::Float => (Some(Value::Integer(53)), None), // Double precision
                    _ => (None, None),
                };

                rows.push(Row::from_values(vec![
                    Value::Text(Arc::from("def")),                // catalog
                    Value::Text(Arc::from(schema_name.as_str())), // schema
                    Value::Text(Arc::from(actual_table_name.as_str())),
                    Value::Text(Arc::from(col.name.as_str())),
                    Value::Integer(ordinal),
                    match default_value {
                        Some(s) => Value::Text(Arc::from(s.as_str())),
                        None => Value::Null(DataType::Text),
                    },
                    Value::Text(Arc::from(is_nullable)),
                    Value::Text(Arc::from(data_type.as_str())),
                    char_max_len.unwrap_or(Value::Null(DataType::Integer)),
                    num_precision.unwrap_or(Value::Null(DataType::Integer)),
                    num_scale.unwrap_or(Value::Null(DataType::Integer)),
                ]));
            }
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Build information_schema.functions result
    fn build_functions_result(&self) -> Result<Box<dyn QueryResult>> {
        // Get function lists from function registry
        let function_registry = self.function_registry();
        let scalar_functions = function_registry.list_scalars();
        let aggregate_functions = function_registry.list_aggregates();
        let window_functions = function_registry.list_windows();

        // Columns: function_catalog, function_schema, function_name, function_type, data_type, is_deterministic
        let columns = vec![
            "function_catalog".to_string(),
            "function_schema".to_string(),
            "function_name".to_string(),
            "function_type".to_string(),
            "data_type".to_string(),
            "is_deterministic".to_string(),
        ];

        let mut rows: Vec<Row> = Vec::new();

        // Helper function to convert FunctionDataType to string
        fn function_data_type_to_string(dtype: &crate::functions::FunctionDataType) -> String {
            match dtype {
                crate::functions::FunctionDataType::Any => "ANY".to_string(),
                crate::functions::FunctionDataType::Integer => "INTEGER".to_string(),
                crate::functions::FunctionDataType::Float => "FLOAT".to_string(),
                crate::functions::FunctionDataType::String => "TEXT".to_string(),
                crate::functions::FunctionDataType::Boolean => "BOOLEAN".to_string(),
                crate::functions::FunctionDataType::Timestamp => "TIMESTAMP".to_string(),
                crate::functions::FunctionDataType::Date => "DATE".to_string(),
                crate::functions::FunctionDataType::Time => "TIME".to_string(),
                crate::functions::FunctionDataType::DateTime => "TIMESTAMP".to_string(),
                crate::functions::FunctionDataType::Json => "JSON".to_string(),
                crate::functions::FunctionDataType::Unknown => "UNKNOWN".to_string(),
            }
        }

        // Add scalar functions
        for func_name in scalar_functions {
            if let Some(func_info) = function_registry.get_info(&func_name) {
                let return_type_str =
                    function_data_type_to_string(&func_info.signature.return_type);
                rows.push(Row::from_values(vec![
                    Value::Text(Arc::from("def")),
                    Value::Text(Arc::from("sys")),
                    Value::Text(Arc::from(func_name.as_str())),
                    Value::Text(Arc::from("SCALAR")),
                    Value::Text(Arc::from(return_type_str.as_str())),
                    Value::Boolean(true), // Built-in functions are deterministic
                ]));
            }
        }

        // Add aggregate functions
        for func_name in aggregate_functions {
            if let Some(func_info) = function_registry.get_info(&func_name) {
                let return_type_str =
                    function_data_type_to_string(&func_info.signature.return_type);
                rows.push(Row::from_values(vec![
                    Value::Text(Arc::from("def")),
                    Value::Text(Arc::from("sys")),
                    Value::Text(Arc::from(func_name.as_str())),
                    Value::Text(Arc::from("AGGREGATE")),
                    Value::Text(Arc::from(return_type_str.as_str())),
                    Value::Boolean(true),
                ]));
            }
        }

        // Add window functions
        for func_name in window_functions {
            if let Some(func_info) = function_registry.get_info(&func_name) {
                let return_type_str =
                    function_data_type_to_string(&func_info.signature.return_type);
                rows.push(Row::from_values(vec![
                    Value::Text(Arc::from("def")),
                    Value::Text(Arc::from("sys")),
                    Value::Text(Arc::from(func_name.as_str())),
                    Value::Text(Arc::from("WINDOW")),
                    Value::Text(Arc::from(return_type_str.as_str())),
                    Value::Boolean(true),
                ]));
            }
        }

        // Add user-defined functions
        // Query _sys_functions for user functions, if the table exists
        let sql = "SELECT schema, name, parameters, return_type FROM _sys_functions ORDER BY name";
        let mut parser = Parser::new(sql);
        if let Ok(program) = parser.parse_program() {
            if let Some(Statement::Select(stmt)) = program.statements.into_iter().next() {
                if let Ok(mut result) = self.execute_select(&stmt, &ExecutionContext::default()) {
                    while result.next() {
                        let row = result.row();
                        if let (
                            Some(schema_val),
                            Some(Value::Text(name)),
                            Some(Value::Json(_params)),
                            Some(Value::Text(return_type_str)),
                        ) = (row.get(0), row.get(1), row.get(2), row.get(3))
                        {
                            let function_schema = match schema_val {
                                Value::Text(s) => s.clone(),
                                Value::Null(_) => Arc::from("public"),
                                _ => Arc::from("public"),
                            };
                            // Map return_type string to data_type
                            let data_type = match return_type_str.as_ref() {
                                "Integer" => "INTEGER",
                                "Text" => "TEXT",
                                "Boolean" => "BOOLEAN",
                                "Float" => "FLOAT",
                                "Timestamp" => "TIMESTAMP",
                                "Date" => "DATE",
                                "Time" => "TIME",
                                "Json" => "JSON",
                                _ => "UNKNOWN",
                            };
                            rows.push(Row::from_values(vec![
                                Value::Text(Arc::from("def")),
                                Value::Text(function_schema),
                                Value::Text(name.clone()),
                                Value::Text(Arc::from("SCALAR")),
                                Value::Text(Arc::from(data_type)),
                                Value::Boolean(true),
                            ]));
                        }
                    }
                }
            }
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Build information_schema.views result
    fn build_views_result(&self) -> Result<Box<dyn QueryResult>> {
        let view_names = self.engine.list_views()?;

        // Columns: table_catalog, table_schema, table_name, view_definition
        let columns = vec![
            "table_catalog".to_string(),
            "table_schema".to_string(),
            "table_name".to_string(),
            "view_definition".to_string(),
        ];

        let mut rows: Vec<Row> = Vec::new();

        for view_name in view_names {
            if let Ok(Some(view_def)) = self.engine.get_view(&view_name) {
                rows.push(Row::from_values(vec![
                    Value::Text(Arc::from("def")),
                    Value::Null(DataType::Text),
                    Value::Text(Arc::from(view_def.original_name.as_str())),
                    Value::Text(Arc::from(view_def.query.as_str())),
                ]));
            }
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Build information_schema.statistics result (indexes)
    fn build_statistics_result(&self) -> Result<Box<dyn QueryResult>> {
        let tx = self.engine.begin_transaction()?;
        let table_names = tx.list_tables()?;

        // Columns: table_catalog, table_schema, table_name, index_name, seq_in_index,
        //          column_name, non_unique, index_type
        let columns = vec![
            "table_catalog".to_string(),
            "table_schema".to_string(),
            "table_name".to_string(),
            "index_name".to_string(),
            "seq_in_index".to_string(),
            "column_name".to_string(),
            "non_unique".to_string(),
            "index_type".to_string(),
        ];

        let mut rows: Vec<Row> = Vec::new();

        for table_name in table_names {
            let table = tx.get_table(&table_name)?;

            // Parse qualified table names
            let (schema_name, actual_table_name) = if let Some(dot_pos) = table_name.find('.') {
                let schema = &table_name[..dot_pos];
                let name = &table_name[dot_pos + 1..];
                (schema.to_string(), name.to_string())
            } else {
                ("public".to_string(), table_name.clone())
            };

            // Get indexes for this table
            if let Ok(indexes) = self.engine.list_table_indexes(&table_name) {
                for index_name in indexes.keys() {
                    if let Some(index) = table.get_index(index_name) {
                        let column_names = index.column_names();
                        let is_unique = index.is_unique();
                        let index_type = index.index_type().as_str().to_uppercase();

                        // Add a row for each column in the index
                        for (seq, col_name) in column_names.iter().enumerate() {
                            rows.push(Row::from_values(vec![
                                Value::Text(Arc::from("def")),
                                Value::Text(Arc::from(schema_name.as_str())),
                                Value::Text(Arc::from(actual_table_name.as_str())),
                                Value::Text(Arc::from(index_name.as_str())),
                                Value::Integer((seq + 1) as i64),
                                Value::Text(Arc::from(col_name.as_str())),
                                Value::Boolean(!is_unique),
                                Value::Text(Arc::from(index_type.as_str())),
                            ]));
                        }
                    }
                }
            }
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Build information_schema.sequences result (empty - no sequences supported)
    fn build_sequences_result(&self) -> Result<Box<dyn QueryResult>> {
        // Columns: sequence_catalog, sequence_schema, sequence_name, data_type, etc.
        // Since Oxibase doesn't support sequences, return empty result
        let columns = vec![
            "sequence_catalog".to_string(),
            "sequence_schema".to_string(),
            "sequence_name".to_string(),
            "data_type".to_string(),
            "numeric_precision".to_string(),
            "numeric_scale".to_string(),
            "start_value".to_string(),
            "minimum_value".to_string(),
            "maximum_value".to_string(),
            "increment".to_string(),
            "cycle_option".to_string(),
        ];

        let rows: Vec<Row> = Vec::new();

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }
}
