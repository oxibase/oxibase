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

//! DDL Statement Execution
//!
//! This module implements execution of Data Definition Language (DDL) statements:
//! - CREATE TABLE
//! - DROP TABLE
//! - CREATE INDEX
//! - DROP INDEX
//! - ALTER TABLE
//! - CREATE VIEW
//! - DROP VIEW

use crate::core::{DataType, Error, Result, Row, SchemaBuilder, Value};
use crate::functions::backends::{
    boa::BoaBackend, python::PythonBackend, rhai::RhaiBackend, ScriptingBackend,
};
use crate::functions::{FunctionDataType, FunctionSignature};
use crate::parser::ast::*;
use crate::storage::functions::{
    StoredFunction, StoredParameter, CREATE_FUNCTIONS_SQL, SYS_FUNCTIONS,
};
use crate::storage::traits::{result::EmptyResult, Engine, QueryResult};
use crate::storage::{
    is_stored_functions_table, StoredScriptFunction, CREATE_STORED_FUNCTIONS_SQL, STORED_FUNCTIONS,
};
use rustc_hash::FxHashMap;

use serde_json;
use std::sync::Arc;

use super::context::ExecutionContext;
use super::expression::ExpressionEval;
use super::result::ExecResult;
use super::Executor;

impl Executor {
    /// Execute a CREATE TABLE statement
    pub(crate) fn execute_create_table(
        &self,
        stmt: &CreateTableStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let table_name = &stmt.table_name.value();

        // Check if schema exists for qualified table names
        if let Some(schema_name) = stmt.table_name.schema() {
            let schemas = self.engine.schemas.read().unwrap();
            if !schemas.contains_key(&schema_name.to_lowercase()) {
                return Err(Error::SchemaNotFound(schema_name));
            }
        }

        // Check if table already exists
        if self.engine.table_exists(table_name)? {
            if stmt.if_not_exists {
                return Ok(Box::new(ExecResult::empty()));
            }
            return Err(Error::TableExists(table_name.clone()));
        }

        // Check if a view with the same name exists
        if self.engine.view_exists(table_name)? {
            return Err(Error::internal(format!(
                "cannot create table '{}': a view with the same name exists",
                table_name
            )));
        }

        // Handle CREATE TABLE ... AS SELECT ...
        if let Some(ref select_stmt) = stmt.as_select {
            return self.execute_create_table_as_select(
                table_name,
                select_stmt,
                stmt.if_not_exists,
                ctx,
            );
        }

        // Build schema from column definitions
        let mut schema_builder = SchemaBuilder::new(table_name);

        // Collect columns with UNIQUE constraints to create indexes after table creation
        let mut unique_columns: Vec<String> = Vec::new();

        for col_def in &stmt.columns {
            let col_name = &col_def.name.value;
            let data_type = self.parse_data_type(&col_def.data_type)?;
            let nullable = !col_def
                .constraints
                .iter()
                .any(|c| matches!(c, ColumnConstraint::NotNull));
            let is_primary_key = col_def
                .constraints
                .iter()
                .any(|c| matches!(c, ColumnConstraint::PrimaryKey));

            // Validate PRIMARY KEY type - only INTEGER is supported
            if is_primary_key && data_type != DataType::Integer {
                return Err(Error::ParseError(format!(
                    "PRIMARY KEY column '{}' must be INTEGER type, got {:?}. Only INTEGER PRIMARY KEY is supported.",
                    col_name, data_type
                )));
            }

            let is_unique = col_def
                .constraints
                .iter()
                .any(|c| matches!(c, ColumnConstraint::Unique));

            let is_auto_increment = col_def
                .constraints
                .iter()
                .any(|c| matches!(c, ColumnConstraint::AutoIncrement));

            // Extract DEFAULT expression
            let default_expr = col_def.constraints.iter().find_map(|c| {
                if let ColumnConstraint::Default(expr) = c {
                    Some(format!("{}", expr))
                } else {
                    None
            }
        });

        let mut scope = rhai::Scope::new();

        // Create arguments array for compatibility
        let mut args_array = rhai::Array::new();
        for arg in args {
            match arg {
                Value::Integer(i) => args_array.push(rhai::Dynamic::from(*i)),
                Value::Float(f) => args_array.push(rhai::Dynamic::from(*f)),
                Value::Text(s) => args_array.push(rhai::Dynamic::from(s.as_ref().to_string())),
                Value::Boolean(b) => args_array.push(rhai::Dynamic::from(*b)),
                _ => return Err(Error::internal("Unsupported argument type for Rhai")),
            };
        }
        scope.push("arguments", args_array);

        // Bind arguments to scope using parameter names
        for (i, arg) in args.iter().enumerate() {
            let var_name = param_names[i];
            match arg {
                Value::Integer(i) => scope.push(var_name, *i),
                Value::Float(f) => scope.push(var_name, *f),
                Value::Text(s) => scope.push(var_name, s.as_ref().to_string()),
                Value::Boolean(b) => scope.push(var_name, *b),
                _ => return Err(Error::internal("Unsupported argument type for Rhai")),
            };
        }

        // Execute the script
        match engine.eval_with_scope::<rhai::Dynamic>(&mut scope, &stored_function.code) {
            Ok(result) => {
                // Convert Rhai Dynamic to Value
                if result.is::<i64>() {
                    Ok(Value::Integer(*result.downcast_ref::<i64>().unwrap()))
                } else if result.is::<f64>() {
                    Ok(Value::Float(*result.downcast_ref::<f64>().unwrap()))
                } else if result.is::<String>() {
                    Ok(Value::Text(std::sync::Arc::from(result.downcast_ref::<String>().unwrap().as_str())))
                } else if result.is::<bool>() {
                    Ok(Value::Boolean(*result.downcast_ref::<bool>().unwrap()))
                } else {
                    Ok(Value::Text(std::sync::Arc::from(format!("{:?}", result))))
                }
            }
            Err(e) => Err(Error::internal(format!("Rhai execution error: {}", e))),
        }
    }

    /// Execute a USE SCHEMA statement
    pub(crate) fn execute_use_schema(
        &self,
        _stmt: &UseSchemaStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        // TODO: Implement schema switching - for now, just succeed
        Ok(Box::new(ExecResult::empty()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::mvcc::engine::MVCCEngine;
    use std::sync::Arc;

    fn create_test_executor() -> Executor {
        let engine = MVCCEngine::in_memory();
        engine.open_engine().unwrap();
        Executor::new(Arc::new(engine))
    }

    #[test]
    fn test_create_table() {
        let executor = create_test_executor();

        let result = executor
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .unwrap();
        assert_eq!(result.rows_affected(), 0);

        // Verify table exists
        assert!(executor.engine().table_exists("users").unwrap());
    }

    #[test]
    fn test_create_table_if_not_exists() {
        let executor = create_test_executor();

        executor
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY)")
            .unwrap();

        // Should not error with IF NOT EXISTS
        let result = executor
            .execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY)")
            .unwrap();
        assert_eq!(result.rows_affected(), 0);
    }

    #[test]
    fn test_create_table_already_exists() {
        let executor = create_test_executor();

        executor
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY)")
            .unwrap();

        // Should error without IF NOT EXISTS
        let result = executor.execute("CREATE TABLE users (id INTEGER PRIMARY KEY)");
        assert!(result.is_err());
    }

    #[test]
    fn test_drop_table() {
        let executor = create_test_executor();

        executor
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY)")
            .unwrap();
        assert!(executor.engine().table_exists("users").unwrap());

        executor.execute("DROP TABLE users").unwrap();
        assert!(!executor.engine().table_exists("users").unwrap());
    }

    #[test]
    fn test_drop_table_if_exists() {
        let executor = create_test_executor();

        // Should not error with IF EXISTS
        let result = executor
            .execute("DROP TABLE IF EXISTS nonexistent")
            .unwrap();
        assert_eq!(result.rows_affected(), 0);
    }

    #[test]
    fn test_drop_table_not_found() {
        let executor = create_test_executor();

        let result = executor.execute("DROP TABLE nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_index() {
        let executor = create_test_executor();

        executor
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
            .unwrap();

        let result = executor
            .execute("CREATE INDEX idx_name ON users (name)")
            .unwrap();
        assert_eq!(result.rows_affected(), 0);
    }

    #[test]
    fn test_create_unique_index() {
        let executor = create_test_executor();

        executor
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT)")
            .unwrap();

        let result = executor
            .execute("CREATE UNIQUE INDEX idx_email ON users (email)")
            .unwrap();
        assert_eq!(result.rows_affected(), 0);
    }

    #[test]
    fn test_parse_data_type() {
        let executor = create_test_executor();

        assert_eq!(
            executor.parse_data_type("INTEGER").unwrap(),
            DataType::Integer
        );
        assert_eq!(executor.parse_data_type("INT").unwrap(), DataType::Integer);
        assert_eq!(
            executor.parse_data_type("BIGINT").unwrap(),
            DataType::Integer
        );
        assert_eq!(executor.parse_data_type("FLOAT").unwrap(), DataType::Float);
        assert_eq!(executor.parse_data_type("DOUBLE").unwrap(), DataType::Float);
        assert_eq!(executor.parse_data_type("TEXT").unwrap(), DataType::Text);
        assert_eq!(
            executor.parse_data_type("VARCHAR(255)").unwrap(),
            DataType::Text
        );
        assert_eq!(
            executor.parse_data_type("BOOLEAN").unwrap(),
            DataType::Boolean
        );
        assert_eq!(
            executor.parse_data_type("TIMESTAMP").unwrap(),
            DataType::Timestamp
        );
        assert_eq!(executor.parse_data_type("JSON").unwrap(), DataType::Json);
    }
}
