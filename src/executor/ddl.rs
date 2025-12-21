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

use crate::core::{DataType, Error, Result, SchemaBuilder, Value};
use crate::functions::{global_registry, FunctionSignature, FunctionDataType};
use crate::parser::ast::*;
use crate::storage::traits::{Engine, QueryResult, result::EmptyResult};
use rustc_hash::FxHashMap;

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

            // Extract CHECK expression
            let check_expr = col_def.constraints.iter().find_map(|c| {
                if let ColumnConstraint::Check(expr) = c {
                    Some(format!("{}", expr))
                } else {
                    None
                }
            });

            // Use add_with_constraints to include DEFAULT and CHECK
            schema_builder = schema_builder.add_with_constraints(
                col_name,
                data_type,
                nullable && !is_primary_key,
                is_primary_key,
                is_auto_increment,
                default_expr,
                check_expr,
            );

            // Track UNIQUE columns for index creation
            if is_unique && !is_primary_key {
                unique_columns.push(col_name.clone());
            }
        }

        let schema = schema_builder.build();

        // Collect table-level UNIQUE constraints (multi-column unique indexes)
        let mut table_unique_constraints: Vec<Vec<String>> = Vec::new();
        for constraint in &stmt.table_constraints {
            if let TableConstraint::Unique(cols) = constraint {
                let col_names: Vec<String> = cols.iter().map(|c| c.value.clone()).collect();
                table_unique_constraints.push(col_names);
            }
            // Note: TableConstraint::Check is not yet supported at schema level
            // Note: TableConstraint::PrimaryKey for composite keys is not yet supported
        }

        // Check if there's an active transaction
        let mut active_tx = self.active_transaction.lock().unwrap();

        if let Some(ref mut _tx_state) = *active_tx {
            // Transactional DDL: Create table immediately but log for undo
            self.engine.create_table(schema.clone())?;

            if let Some(ref mut tx_state) = *active_tx {
                tx_state
                    .ddl_undo_log
                    .push(super::DeferredDdlOperation::CreateTable {
                        name: table_name.clone(),
                    });
            }

            // Create indexes within the same transaction context if possible,
            // but for now we'll do them separately or assume they are part of the table creation
            // Note: Since we use engine.create_table directly, it's global.
            // But we need to make sure indexes are also undone if we rollback.
        } else {
            // No active transaction - use direct engine call (auto-committed)
            self.engine.create_table(schema)?;
        }

        // Create unique indexes for columns with UNIQUE constraint (shared logic)
        let needs_indexes = !unique_columns.is_empty() || !table_unique_constraints.is_empty();
        if needs_indexes {
            let mut tx = self.engine.begin_transaction()?;
            let table = tx.get_table(table_name)?;

            for col_name in &unique_columns {
                let index_name = format!("unique_{}_{}", table_name, col_name);
                table.create_index(&index_name, &[col_name.as_str()], true)?;
                // Get index type for WAL persistence
                let idx_type = table
                    .get_index(&index_name)
                    .map(|idx| idx.index_type())
                    .unwrap_or(crate::core::IndexType::BTree);
                // Record index creation to WAL for persistence
                self.engine.record_create_index(
                    table_name,
                    &index_name,
                    std::slice::from_ref(col_name),
                    true,
                    idx_type,
                );
            }

            // Create multi-column unique indexes from table-level constraints
            for (i, col_names) in table_unique_constraints.iter().enumerate() {
                let index_name = format!("unique_{}_{}", table_name, i);
                let col_refs: Vec<&str> = col_names.iter().map(|s| s.as_str()).collect();
                table.create_index(&index_name, &col_refs, true)?;
                // Get index type for WAL persistence
                let idx_type = table
                    .get_index(&index_name)
                    .map(|idx| idx.index_type())
                    .unwrap_or(crate::core::IndexType::BTree);
                // Record index creation to WAL for persistence
                self.engine
                    .record_create_index(table_name, &index_name, col_names, true, idx_type);
            }

            tx.commit()?;
        }

        Ok(Box::new(ExecResult::empty()))
    }

    /// Execute CREATE TABLE ... AS SELECT ...
    fn execute_create_table_as_select(
        &self,
        table_name: &str,
        select_stmt: &SelectStatement,
        _if_not_exists: bool,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        use crate::core::Row;

        // Execute the SELECT query to get the result
        // Use execute_select for full query processing (DISTINCT, ORDER BY, etc.)
        let mut result = self.execute_select(select_stmt, ctx)?;
        let columns: Vec<String> = result.columns().to_vec();

        // Materialize the result to get the rows
        let mut rows: Vec<Row> = Vec::new();
        while result.next() {
            rows.push(result.take_row());
        }

        // Infer schema from the result columns and first row (if available)
        let mut schema_builder = SchemaBuilder::new(table_name);

        for (i, col_name) in columns.iter().enumerate() {
            // Extract base column name (without table prefix)
            let base_name = if let Some(pos) = col_name.rfind('.') {
                &col_name[pos + 1..]
            } else {
                col_name.as_str()
            };

            // Infer data type from first row if available
            let data_type = if let Some(first_row) = rows.first() {
                if let Some(value) = first_row.get(i) {
                    Self::infer_data_type(value)
                } else {
                    DataType::Text // Default to TEXT
                }
            } else {
                DataType::Text // Default to TEXT for empty result
            };

            schema_builder = schema_builder.add_nullable(base_name, data_type);
        }

        let schema = schema_builder.build();

        // Create the table
        self.engine.create_table(schema)?;

        // Insert the rows into the new table
        let rows_count = rows.len();
        if !rows.is_empty() {
            let mut tx = self.engine.begin_transaction()?;
            let mut table = tx.get_table(table_name)?;

            for row in rows {
                let _ = table.insert(row)?;
            }

            // Commit the transaction - it will commit all tables via commit_all_tables()
            tx.commit()?;
        }

        Ok(Box::new(ExecResult::with_rows_affected(rows_count as i64)))
    }

    /// Infer data type from a Value
    fn infer_data_type(value: &crate::core::Value) -> DataType {
        match value {
            crate::core::Value::Integer(_) => DataType::Integer,
            crate::core::Value::Float(_) => DataType::Float,
            crate::core::Value::Text(_) => DataType::Text,
            crate::core::Value::Boolean(_) => DataType::Boolean,
            crate::core::Value::Timestamp(_) => DataType::Timestamp,
            crate::core::Value::Json(_) => DataType::Json,
            crate::core::Value::Null(_) => DataType::Text, // Default nulls to TEXT
        }
    }

    /// Execute a DROP TABLE statement
    pub(crate) fn execute_drop_table(
        &self,
        stmt: &DropTableStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let table_name = &stmt.table_name.value();

        // Check if table exists
        if !self.engine.table_exists(table_name)? {
            if stmt.if_exists {
                return Ok(Box::new(ExecResult::empty()));
            }
            return Err(Error::TableNotFoundByName(table_name.clone()));
        }

        // Check if there's an active transaction
        let mut active_tx = self.active_transaction.lock().unwrap();

        if let Some(ref mut _tx_state) = *active_tx {
            // Transactional DDL: Get schema before dropping, then drop immediately
            let schema = self.engine.get_table_schema(table_name)?;
            self.engine.drop_table_internal(table_name)?;

            if let Some(ref mut tx_state) = *active_tx {
                tx_state
                    .ddl_undo_log
                    .push(super::DeferredDdlOperation::DropTable {
                        name: table_name.clone(),
                        schema,
                    });
            }

            eprintln!(
                "Warning: DROP TABLE '{}' within transaction - data cannot be recovered on rollback",
                table_name
            );
        } else {
            // No active transaction - use engine method directly (auto-committed with WAL)
            self.engine.drop_table_internal(table_name)?;
        }

        Ok(Box::new(ExecResult::empty()))
    }

    /// Execute a CREATE INDEX statement
    pub(crate) fn execute_create_index(
        &self,
        stmt: &CreateIndexStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        // Check if schema exists for qualified table names
        if let Some(schema) = stmt.table_name.schema() {
            let schemas = self.engine.schemas.read().unwrap();
            if !schemas.contains_key(&schema.to_lowercase()) {
                return Err(Error::SchemaNotFound(schema));
            }
        }

        let table_name = &stmt.table_name.value();
        let index_name = &stmt.index_name.value;

        // Check if table exists
        if !self.engine.table_exists(table_name)? {
            return Err(Error::TableNotFoundByName(table_name.clone()));
        }

        // Check if index already exists
        if self.engine.index_exists(index_name, table_name)? {
            if stmt.if_not_exists {
                return Ok(Box::new(ExecResult::empty()));
            }
            return Err(Error::internal(format!(
                "index already exists: {}",
                index_name
            )));
        }

        // Determine index type
        let is_unique = stmt.is_unique;

        // Get table to validate columns exist
        let tx = self.engine.begin_transaction()?;
        let table = tx.get_table(table_name)?;
        let schema = table.schema();

        // Validate columns
        for col_id in &stmt.columns {
            let col_name = &col_id.value;
            if !schema
                .columns
                .iter()
                .any(|c| c.name.eq_ignore_ascii_case(col_name))
            {
                return Err(Error::ColumnNotFoundNamed(col_name.clone()));
            }
        }

        // Collect column names
        let column_names: Vec<String> = stmt.columns.iter().map(|c| c.value.clone()).collect();
        let column_refs: Vec<&str> = column_names.iter().map(|s| s.as_str()).collect();

        // Check if IF NOT EXISTS should suppress errors:
        // 1. Index with same name already exists, OR
        // 2. An index already exists on the column(s) - this prevents errors like
        //    "cannot create non-unique index when unique already exists"
        if stmt.if_not_exists {
            // Check by name
            if table.get_index(index_name).is_some() {
                return Ok(Box::new(ExecResult::empty()));
            }
            // For single-column indexes, also check if column already has an index
            if column_names.len() == 1 && table.has_index_on_column(&column_names[0]) {
                return Ok(Box::new(ExecResult::empty()));
            }
        }

        // Convert USING clause IndexMethod to core IndexType
        let requested_index_type = stmt.index_method.map(|method| match method {
            crate::parser::ast::IndexMethod::BTree => crate::core::IndexType::BTree,
            crate::parser::ast::IndexMethod::Hash => crate::core::IndexType::Hash,
            crate::parser::ast::IndexMethod::Bitmap => crate::core::IndexType::Bitmap,
        });

        // Create the index (supports both single and multi-column)
        // Use create_index_with_type to pass the optional explicit index type
        table.create_index_with_type(index_name, &column_refs, is_unique, requested_index_type)?;

        // Get the created index to determine its actual type for WAL persistence
        let index_type = table
            .get_index(index_name)
            .map(|idx| idx.index_type())
            .unwrap_or(crate::core::IndexType::BTree);

        // Record index creation to WAL for persistence
        self.engine.record_create_index(
            table_name,
            index_name,
            &column_names,
            is_unique,
            index_type,
        );

        Ok(Box::new(ExecResult::empty()))
    }

    /// Execute a DROP INDEX statement
    pub(crate) fn execute_drop_index(
        &self,
        stmt: &DropIndexStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let index_name = &stmt.index_name.value;

        // Get table name if specified
        let table_name = match &stmt.table_name {
            Some(t) => t.value.clone(),
            None => {
                return Err(Error::InvalidArgumentMessage(
                    "DROP INDEX requires table name".to_string(),
                ))
            }
        };

        // Check if table exists
        if !self.engine.table_exists(&table_name)? {
            if stmt.if_exists {
                return Ok(Box::new(ExecResult::empty()));
            }
            return Err(Error::TableNotFoundByName(table_name));
        }

        // Check if index exists
        if !self.engine.index_exists(index_name, &table_name)? {
            if stmt.if_exists {
                return Ok(Box::new(ExecResult::empty()));
            }
            return Err(Error::IndexNotFoundByName(index_name.to_string()));
        }

        // Get the table and drop the index
        let tx = self.engine.begin_transaction()?;
        let table = tx.get_table(&table_name)?;
        table.drop_index(index_name)?;

        // Record index drop to WAL for persistence
        self.engine.record_drop_index(&table_name, index_name);

        Ok(Box::new(ExecResult::empty()))
    }

    /// Execute an ALTER TABLE statement
    pub(crate) fn execute_alter_table(
        &self,
        stmt: &AlterTableStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let table_name = &stmt.table_name.value();

        // Check if table exists
        if !self.engine.table_exists(table_name)? {
            return Err(Error::TableNotFoundByName(table_name.clone()));
        }

        // Get the table for modifications
        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(table_name)?;

        // Process the single ALTER TABLE operation
        match stmt.operation {
            AlterTableOperation::AddColumn => {
                if let Some(ref col_def) = stmt.column_def {
                    let data_type = self.parse_data_type(&col_def.data_type)?;
                    let nullable = !col_def
                        .constraints
                        .iter()
                        .any(|c| matches!(c, ColumnConstraint::NotNull));

                    // Extract default expression if present
                    let default_expr = col_def.constraints.iter().find_map(|c| {
                        if let ColumnConstraint::Default(expr) = c {
                            Some(expr.to_string())
                        } else {
                            None
                        }
                    });

                    // Pre-compute the default value for schema evolution (backfilling existing rows)
                    // The default_expr string is also stored for new INSERTs
                    let default_value = if let Some(ref expr_str) = default_expr {
                        let val = self.evaluate_default_expression(expr_str, data_type)?;
                        if val.is_null() {
                            None
                        } else {
                            Some(val)
                        }
                    } else {
                        None
                    };

                    table.create_column_with_default_value(
                        &col_def.name.value,
                        data_type,
                        nullable,
                        default_expr.clone(),
                        default_value,
                    )?;

                    // Record ALTER TABLE ADD COLUMN to WAL for persistence
                    self.engine.record_alter_table_add_column(
                        table_name,
                        &col_def.name.value,
                        data_type,
                        nullable,
                        default_expr.as_deref(),
                    );
                } else {
                    return Err(Error::InvalidArgumentMessage(
                        "ADD COLUMN requires column definition".to_string(),
                    ));
                }
            }
            AlterTableOperation::DropColumn => {
                if let Some(ref col_name) = stmt.column_name {
                    table.drop_column(&col_name.value)?;

                    // Record ALTER TABLE DROP COLUMN to WAL for persistence
                    self.engine
                        .record_alter_table_drop_column(table_name, &col_name.value);
                } else {
                    return Err(Error::InvalidArgumentMessage(
                        "DROP COLUMN requires column name".to_string(),
                    ));
                }
            }
            AlterTableOperation::RenameColumn => match (&stmt.column_name, &stmt.new_column_name) {
                (Some(old_name), Some(new_name)) => {
                    table.rename_column(&old_name.value, &new_name.value)?;

                    // Record ALTER TABLE RENAME COLUMN to WAL for persistence
                    self.engine.record_alter_table_rename_column(
                        table_name,
                        &old_name.value,
                        &new_name.value,
                    );
                }
                _ => {
                    return Err(Error::InvalidArgumentMessage(
                        "RENAME COLUMN requires old and new column names".to_string(),
                    ));
                }
            },
            AlterTableOperation::ModifyColumn => {
                if let Some(ref col_def) = stmt.column_def {
                    let data_type = self.parse_data_type(&col_def.data_type)?;
                    let nullable = !col_def
                        .constraints
                        .iter()
                        .any(|c| matches!(c, ColumnConstraint::NotNull));

                    table.modify_column(&col_def.name.value, data_type, nullable)?;

                    // Record ALTER TABLE MODIFY COLUMN to WAL for persistence
                    self.engine.record_alter_table_modify_column(
                        table_name,
                        &col_def.name.value,
                        data_type,
                        nullable,
                    );
                } else {
                    return Err(Error::InvalidArgumentMessage(
                        "MODIFY COLUMN requires column definition".to_string(),
                    ));
                }
            }
            AlterTableOperation::RenameTable => {
                if let Some(ref new_name) = stmt.new_table_name {
                    tx.rename_table(table_name, &new_name.value)?;

                    // Record ALTER TABLE RENAME TO WAL for persistence
                    self.engine
                        .record_alter_table_rename(table_name, &new_name.value);
                } else {
                    return Err(Error::InvalidArgumentMessage(
                        "RENAME TABLE requires new table name".to_string(),
                    ));
                }
            }
        }

        tx.commit()?;
        Ok(Box::new(ExecResult::empty()))
    }

    /// Execute a CREATE VIEW statement
    pub(crate) fn execute_create_view(
        &self,
        stmt: &CreateViewStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        // Check if schema exists for qualified view names
        if let Some(schema) = stmt.view_name.schema() {
            let schemas = self.engine.schemas.read().unwrap();
            if !schemas.contains_key(&schema.to_lowercase()) {
                return Err(Error::SchemaNotFound(schema));
            }
        }

        let view_name = &stmt.view_name.value();

        // Check if a table with the same name exists
        if self.engine.table_exists(view_name)? {
            return Err(Error::TableAlreadyExists);
        }

        // Convert the query to SQL string
        let query_sql = stmt.query.to_string();

        // Create the view (engine handles if_not_exists logic)
        self.engine
            .create_view(view_name, query_sql, stmt.if_not_exists)?;

        Ok(Box::new(ExecResult::empty()))
    }

    /// Execute a DROP VIEW statement
    pub(crate) fn execute_drop_view(
        &self,
        stmt: &DropViewStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let view_name = &stmt.view_name.value;

        // Drop the view (engine handles if_exists logic)
        self.engine.drop_view(view_name, stmt.if_exists)?;

        Ok(Box::new(ExecResult::empty()))
    }

    /// Execute a CREATE COLUMNAR INDEX statement
    ///
    /// DEPRECATED: The COLUMNAR INDEX syntax is deprecated.
    /// Use `CREATE INDEX` instead - the index type (BTree, Hash, Bitmap)
    /// is automatically selected based on the column data type:
    /// - INTEGER, FLOAT, TIMESTAMP -> BTree (range queries)
    /// - TEXT, JSON -> Hash (O(1) equality lookups)
    /// - BOOLEAN -> Bitmap (low cardinality)
    /// - Multiple columns -> MultiColumn composite index
    pub(crate) fn execute_create_columnar_index(
        &self,
        _stmt: &CreateColumnarIndexStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        Err(Error::internal(
            "CREATE COLUMNAR INDEX syntax is deprecated. Use CREATE INDEX instead - the index type is auto-selected based on column type.",
        ))
    }

    /// Execute a CREATE FUNCTION statement
    pub(crate) fn execute_create_function(
        &self,
        stmt: &CreateFunctionStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        // Register the function in the global registry
        let registry = global_registry();
        registry.register_user_defined(
            stmt.function_name.value.clone(),
            stmt.body.clone(),
            FunctionSignature::new(
                // TODO: Map return_type string to FunctionDataType
                FunctionDataType::Unknown,
                // TODO: Map parameters to FunctionDataType
                vec![],
                stmt.parameters.len(),
                stmt.parameters.len(),
            ),
        )?;

        Ok(Box::new(EmptyResult::new()))
    }

    /// Execute a DROP COLUMNAR INDEX statement
    ///
    /// DEPRECATED: The COLUMNAR INDEX syntax is deprecated.
    /// Use `DROP INDEX index_name ON table_name` instead.
    pub(crate) fn execute_drop_columnar_index(
        &self,
        _stmt: &DropColumnarIndexStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        Err(Error::internal(
            "DROP COLUMNAR INDEX syntax is deprecated. Use DROP INDEX index_name ON table_name instead.",
        ))
    }

    /// Parse a SQL data type string to DataType enum
    pub(crate) fn parse_data_type(&self, type_str: &str) -> Result<DataType> {
        let upper = type_str.to_uppercase();
        let base_type = upper.split('(').next().unwrap_or(&upper);

        match base_type {
            "INTEGER" | "INT" | "BIGINT" | "SMALLINT" | "TINYINT" => Ok(DataType::Integer),
            "FLOAT" | "DOUBLE" | "REAL" | "DECIMAL" | "NUMERIC" => Ok(DataType::Float),
            "TEXT" | "VARCHAR" | "CHAR" | "STRING" | "CLOB" => Ok(DataType::Text),
            "BOOLEAN" | "BOOL" => Ok(DataType::Boolean),
            // Date and time are all stored as Timestamp
            "TIMESTAMP" | "DATETIME" | "DATE" | "TIME" => Ok(DataType::Timestamp),
            "JSON" | "JSONB" => Ok(DataType::Json),
            // Binary data stored as Text (base64 encoded)
            "BLOB" | "BINARY" | "VARBINARY" => Ok(DataType::Text),
            _ => Err(Error::Type(format!("Unknown data type: {}", type_str))),
        }
    }

    /// Evaluate a default expression string and return the resulting Value
    fn evaluate_default_expression(
        &self,
        default_expr: &str,
        target_type: DataType,
    ) -> Result<Value> {
        use crate::parser::parse_sql;

        // Parse the default expression as a SELECT expression
        let sql = format!("SELECT {}", default_expr);
        let stmts = match parse_sql(&sql) {
            Ok(s) => s,
            Err(_) => return Ok(Value::null(target_type)),
        };
        if stmts.is_empty() {
            return Ok(Value::null(target_type));
        }

        // Extract the expression from the SELECT statement
        if let Statement::Select(select) = &stmts[0] {
            if let Some(expr) = select.columns.first() {
                let mut eval = ExpressionEval::compile(expr, &[])?;
                let value = eval.eval_slice(&[])?;
                return Ok(value.into_coerce_to_type(target_type));
            }
        }

        Ok(Value::null(target_type))
    }

    /// Execute a CREATE SCHEMA statement
    pub(crate) fn execute_create_schema(
        &self,
        stmt: &CreateSchemaStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let schema_name = stmt.schema_name.value.to_lowercase();

        // Check if schema already exists
        {
            let schemas = self.engine.schemas.read().unwrap();
            if schemas.contains_key(&schema_name) {
                if stmt.if_not_exists {
                    return Ok(Box::new(ExecResult::empty()));
                }
                return Err(Error::SchemaAlreadyExists);
            }
        }

        // Check active transaction
        let mut active_tx = self.active_transaction.lock().unwrap();

        if let Some(ref mut tx_state) = *active_tx {
            // Add to schemas
            {
                let mut schemas = self.engine.schemas.write().unwrap();
                schemas.insert(schema_name.clone(), FxHashMap::default());
            }

            // Add to undo log
            tx_state
                .ddl_undo_log
                .push(super::DeferredDdlOperation::CreateSchema { name: schema_name });
        } else {
            // Add to schemas
            {
                let mut schemas = self.engine.schemas.write().unwrap();
                schemas.insert(schema_name, FxHashMap::default());
            }
        }

        Ok(Box::new(ExecResult::empty()))
    }

    /// Execute a DROP SCHEMA statement
    pub(crate) fn execute_drop_schema(
        &self,
        stmt: &DropSchemaStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let schema_name = stmt.schema_name.value.to_lowercase();

        // Collect tables in schema
        let tables = if let Some(ref tx_state) = *self.active_transaction.lock().unwrap() {
            tx_state.transaction.list_tables()?
        } else {
            let mut tx = self.engine.begin_transaction()?;
            let t = tx.list_tables()?;
            tx.commit()?;
            t
        };
        let mut tables_to_drop = Vec::new();
        for table_name in tables {
            if table_name.starts_with(&format!("{}.", schema_name)) {
                let schema = self.engine.get_table_schema(&table_name)?;
                tables_to_drop.push((table_name, schema));
            }
        }

        // Check active transaction
        let mut active_tx = self.active_transaction.lock().unwrap();

        if let Some(ref mut tx_state) = *active_tx {
            // Drop tables
            for (table_name, _) in &tables_to_drop {
                self.engine.drop_table_internal(table_name)?;
            }

            // Drop schema
            {
                let mut schemas = self.engine.schemas.write().unwrap();
                schemas.remove(&schema_name);
            }

            // Add to undo log
            tx_state
                .ddl_undo_log
                .push(super::DeferredDdlOperation::DropSchema {
                    name: schema_name,
                    tables: tables_to_drop,
                });
        } else {
            // Drop tables in transaction
            {
                let mut tx = self.engine.begin_transaction()?;
                for (table_name, _) in &tables_to_drop {
                    tx.drop_table(table_name)?;
                }
                tx.commit()?;
            }

            // Drop schema
            {
                let mut schemas = self.engine.schemas.write().unwrap();
                schemas.remove(&schema_name);
            }
        }

        Ok(Box::new(ExecResult::empty()))
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
