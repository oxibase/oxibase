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
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! SHOW and DESCRIBE statement execution
//!
//! This module handles metadata query commands:
//! - SHOW TABLES
//! - SHOW VIEWS
//! - SHOW CREATE TABLE
//! - SHOW CREATE VIEW
//! - SHOW INDEXES
//! - DESCRIBE

use std::sync::Arc;

use crate::core::{Error, Result, Row, Value};
use crate::parser::{ast::*, Parser};
use crate::storage::functions::StoredParameter;

use crate::storage::traits::{Engine, QueryResult};

use super::context::ExecutionContext;
use super::result::ExecutorMemoryResult;
use super::Executor;

impl Executor {
    /// Execute SHOW TABLES statement
    pub(crate) fn execute_show_tables(
        &self,
        _stmt: &ShowTablesStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let sql = "SELECT table_name FROM information_schema.tables ORDER BY table_name";
        let mut parser = Parser::new(sql);
        let program = parser.parse_program().map_err(|e| Error::Parse {
            message: format!("Failed to parse internal query: {}", e),
        })?;
        if let Some(Statement::Select(stmt)) = program.statements.into_iter().next() {
            let result = self.execute_select(&stmt, ctx)?;
            // The result already has the correct columns and rows
            Ok(result)
        } else {
            Err(Error::Internal {
                message: "Failed to parse SHOW TABLES query".to_string(),
            })
        }
    }

    /// Execute SHOW VIEWS statement
    pub(crate) fn execute_show_views(
        &self,
        _stmt: &ShowViewsStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let sql = "SELECT table_name FROM information_schema.views ORDER BY table_name";
        let mut parser = Parser::new(sql);
        let program = parser.parse_program().map_err(|e| Error::Parse {
            message: format!("Failed to parse internal query: {}", e),
        })?;
        if let Some(Statement::Select(stmt)) = program.statements.into_iter().next() {
            let mut result = self.execute_select(&stmt, ctx)?;
            // Rename column to view_name
            // But since it's internal, and result has table_name, but SHOW expects view_name
            // For simplicity, since information_schema.views has table_name for views, and SHOW shows view_name, it's the same.
            // But to match, perhaps wrap.
            // For now, since table_name is the view name, and column is table_name, but SHOW has view_name.
            // In the test, SHOW VIEWS has view_name.
            // To match, I can change the column name.
            // But since it's a shortcut, and user expects view_name, let's rename the column.
            // But ExecutorMemoryResult has columns, but since it's dyn QueryResult, hard to modify.
            // Perhaps keep as is, but since the output is the same, and CLI shows the column name.
            // In the code, columns = vec!["view_name".to_string()], but now it's table_name.
            // Problem.
            // To fix, I need to post-process.
            // Collect the rows, change column to "view_name"
            let mut all_rows = Vec::new();
            while result.next() {
                all_rows.push(result.row().clone());
            }
            let columns = vec!["view_name".to_string()];
            let rows = all_rows
                .into_iter()
                .map(|row| {
                    // Assume row has one value, table_name
                    Row::from_values(vec![row.get(0).unwrap().clone()])
                })
                .collect();
            Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
        } else {
            Err(Error::Internal {
                message: "Failed to parse SHOW VIEWS query".to_string(),
            })
        }
    }

    /// Execute SHOW CREATE TABLE statement
    pub(crate) fn execute_show_create_table(
        &self,
        stmt: &ShowCreateTableStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let table_name = &stmt.table_name.value();
        let tx = self.engine.begin_transaction()?;
        let table = tx.get_table(table_name)?;
        let schema = table.schema();

        // Get unique column names from indexes
        let mut unique_columns: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        if let Ok(indexes) = self.engine.list_table_indexes(table_name) {
            for index_name in indexes.keys() {
                if let Some(index) = table.get_index(index_name) {
                    // Only single-column unique indexes should be shown as UNIQUE constraint on column
                    let col_names = index.column_names();
                    if index.is_unique() && col_names.len() == 1 {
                        unique_columns.insert(col_names[0].to_lowercase());
                    }
                }
            }
        }

        // Build CREATE TABLE statement
        let mut create_sql = format!("CREATE TABLE {} (", table_name);
        let col_defs: Vec<String> = schema
            .columns
            .iter()
            .map(|col| {
                let mut def = format!("{} {:?}", col.name, col.data_type);
                if col.primary_key {
                    def.push_str(" PRIMARY KEY");
                    if col.auto_increment {
                        def.push_str(" AUTO_INCREMENT");
                    }
                } else {
                    // Check if this column has a UNIQUE constraint
                    if unique_columns.contains(&col.name.to_lowercase()) {
                        def.push_str(" UNIQUE");
                    }
                    if !col.nullable {
                        def.push_str(" NOT NULL");
                    }
                }
                // Add DEFAULT if present
                if let Some(default_expr) = &col.default_expr {
                    def.push_str(&format!(" DEFAULT {}", default_expr));
                }
                // Add CHECK constraint if present
                if let Some(check) = &col.check_expr {
                    def.push_str(&format!(" CHECK ({})", check));
                }
                def
            })
            .collect();
        create_sql.push_str(&col_defs.join(", "));
        create_sql.push(')');

        let columns = vec!["Table".to_string(), "Create Table".to_string()];
        let rows = vec![Row::from_values(vec![
            Value::Text(Arc::from(table_name.as_str())),
            Value::Text(Arc::from(create_sql.as_str())),
        ])];

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Execute SHOW CREATE VIEW statement
    pub(crate) fn execute_show_create_view(
        &self,
        stmt: &ShowCreateViewStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let view_name = &stmt.view_name.value;

        // Get the view definition
        let view_def = self
            .engine
            .get_view(view_name)?
            .ok_or_else(|| Error::ViewNotFound(view_name.to_string()))?;

        // Build CREATE VIEW statement
        let create_sql = format!(
            "CREATE VIEW {} AS {}",
            view_def.original_name, view_def.query
        );

        let columns = vec!["View".to_string(), "Create View".to_string()];
        let rows = vec![Row::from_values(vec![
            Value::Text(Arc::from(view_def.original_name.as_str())),
            Value::Text(Arc::from(create_sql.as_str())),
        ])];

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Execute SHOW INDEXES statement
    pub(crate) fn execute_show_indexes(
        &self,
        stmt: &ShowIndexesStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let table_name = &stmt.table_name.value();

        // Get a table reference to access indexes
        let tx = self.engine.begin_transaction()?;
        let table = tx.get_table(table_name)?;

        // Get index info from version store through the table
        let index_names = {
            let indexes = self.engine.list_table_indexes(table_name)?;
            indexes.keys().cloned().collect::<Vec<_>>()
        };

        // Build rows with: table_name, index_name, column_name, index_type, is_unique
        let columns = vec![
            "table_name".to_string(),
            "index_name".to_string(),
            "column_name".to_string(),
            "index_type".to_string(),
            "is_unique".to_string(),
        ];

        let mut rows: Vec<Row> = Vec::new();
        for index_name in index_names {
            // Try to get index details from the table's underlying storage
            if let Some(index) = table.get_index(&index_name) {
                let column_names = index.column_names();
                // Show all columns for multi-column indexes
                let column_name = if column_names.len() > 1 {
                    format!("({})", column_names.join(", "))
                } else {
                    column_names
                        .first()
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                };
                let is_unique = index.is_unique();
                let index_type = index.index_type().as_str().to_uppercase();

                rows.push(Row::from_values(vec![
                    Value::Text(Arc::from(table_name.as_str())),
                    Value::Text(Arc::from(index_name.as_str())),
                    Value::Text(Arc::from(column_name.as_str())),
                    Value::Text(Arc::from(index_type)),
                    Value::Boolean(is_unique),
                ]));
            }
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Execute DESCRIBE statement - shows table structure
    pub(crate) fn execute_describe(
        &self,
        stmt: &DescribeStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let table_name = &stmt.table_name.value();
        let tx = self.engine.begin_transaction()?;
        let table = tx.get_table(table_name)?;
        let schema = table.schema();

        // Column headers: Field, Type, Null, Key, Default, Extra
        let columns = vec![
            "Field".to_string(),
            "Type".to_string(),
            "Null".to_string(),
            "Key".to_string(),
            "Default".to_string(),
            "Extra".to_string(),
        ];

        let mut rows: Vec<Row> = Vec::new();
        for col in &schema.columns {
            // Determine type string
            let type_str = format!("{:?}", col.data_type);

            // Determine nullability
            let null_str = if col.nullable { "YES" } else { "NO" };

            // Determine key type
            let key_str = if col.primary_key { "PRI" } else { "" };

            // Get default value if any
            let default_str = col
                .default_expr
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default();

            // Extra info (e.g., auto_increment equivalent)
            let extra_str =
                if col.primary_key && col.data_type == crate::core::types::DataType::Integer {
                    "auto_increment"
                } else {
                    ""
                };

            rows.push(Row::from_values(vec![
                Value::Text(Arc::from(col.name.as_str())),
                Value::Text(Arc::from(type_str.as_str())),
                Value::Text(Arc::from(null_str)),
                Value::Text(Arc::from(key_str)),
                Value::Text(Arc::from(default_str.as_str())),
                Value::Text(Arc::from(extra_str)),
            ]));
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }

    /// Execute SHOW FUNCTIONS statement
    pub(crate) fn execute_show_functions(
        &self,
        stmt: &ShowFunctionsStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let mut rows: Vec<Row> = Vec::new();

        if stmt.plural {
            // SHOW FUNCTIONS - show user-defined functions
            // Ensure functions table exists
            self.ensure_functions_table_exists()?;

            // Select all functions from system table
            let sql = "SELECT name, parameters, return_type, language, code FROM _sys_functions ORDER BY name";
            let mut parser = Parser::new(sql);
            let program = parser.parse_program().map_err(|e| Error::Parse {
                message: format!("Failed to parse internal query: {}", e),
            })?;
            if let Some(Statement::Select(select_stmt)) = program.statements.into_iter().next() {
                let mut result = self.execute_select(&select_stmt, ctx)?;
                while result.next() {
                    let row = result.row();
                    if let (
                        Some(Value::Text(name)),
                        Some(Value::Json(params_json)),
                        Some(Value::Text(return_type)),
                        Some(Value::Text(language)),
                        Some(Value::Text(code)),
                    ) = (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4))
                    {
                        // Parse parameters JSON
                        let parameters: Vec<StoredParameter> = serde_json::from_str(params_json)
                            .map_err(|e| Error::Internal {
                                message: format!("Failed to parse function parameters: {}", e),
                            })?;

                        // Format parameters as "(name type, name type, ...)"
                        let args = if parameters.is_empty() {
                            "()".to_string()
                        } else {
                            format!(
                                "({})",
                                parameters
                                    .iter()
                                    .map(|p| format!("{} {}", p.name, p.data_type))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        };

                        rows.push(Row::from_values(vec![
                            Value::Text(name.clone()),
                            Value::Text(Arc::from(args)),
                            Value::Text(return_type.clone()),
                            Value::Text(language.clone()),
                            Value::Text(code.clone()),
                        ]));
                    }
                }
            }
        } else {
            // SHOW FUNCTION - show built-in functions
            let function_registry = self.function_registry();
            let scalar_functions = function_registry.list_scalars();
            let aggregate_functions = function_registry.list_aggregates();
            let window_functions = function_registry.list_windows();

            // Add scalar functions
            for func_name in scalar_functions {
                rows.push(Row::from_values(vec![
                    Value::Text(Arc::from(func_name.as_str())),
                    Value::Text(Arc::from("SCALAR")),
                ]));
            }

            // Add aggregate functions
            for func_name in aggregate_functions {
                rows.push(Row::from_values(vec![
                    Value::Text(Arc::from(func_name.as_str())),
                    Value::Text(Arc::from("AGGREGATE")),
                ]));
            }

            // Add window functions
            for func_name in window_functions {
                rows.push(Row::from_values(vec![
                    Value::Text(Arc::from(func_name.as_str())),
                    Value::Text(Arc::from("WINDOW")),
                ]));
            }
        }

        let columns = if stmt.plural {
            vec![
                "name".to_string(),
                "args".to_string(),
                "return_type".to_string(),
                "language".to_string(),
                "body".to_string(),
            ]
        } else {
            vec!["name".to_string(), "type".to_string()]
        };

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }
}
