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

use crate::core::{DataType, Error, Result, Row, Schema, SchemaBuilder, Value};
use crate::functions::{FunctionDataType, FunctionSignature};
use crate::parser::ast::*;
use crate::storage::expression::Expression;
use crate::storage::functions::{
    StoredFunction, StoredParameter, CREATE_FUNCTIONS_SQL, SYS_FUNCTIONS,
};
use crate::storage::procedures::{CREATE_PROCEDURES_SQL, SYS_PROCEDURES};
use crate::storage::traits::{result::EmptyResult, Engine, QueryResult};
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

        // Prevent creation of tables in reserved namespaces unless internal
        if Schema::is_reserved_namespace(table_name) && !ctx.is_internal() {
            return Err(Error::ReservedNamespaceModification(table_name.clone()));
        }

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
        let schema_name = stmt
            .table_name
            .schema()
            .unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string())
            .to_lowercase();
        if self.engine.view_exists(&schema_name, table_name)? {
            return Err(Error::ViewAlreadyExists(table_name.clone()));
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

        let mut schema = schema_builder.build();

        // Collect table-level UNIQUE constraints (multi-column unique indexes)
        let mut table_unique_constraints: Vec<Vec<String>> = Vec::new();
        // Collect referenced schemas to update after table creation
        let mut schemas_to_update: Vec<crate::core::Schema> = Vec::new();

        // Process column-level REFERENCES constraints
        for col_def in &stmt.columns {
            let col_name = &col_def.name.value;
            let (col_idx, _) = schema.find_column(col_name).unwrap();

            for constraint in &col_def.constraints {
                if let ColumnConstraint::References(foreign_table, foreign_column) = constraint {
                    let referencing_schema = stmt
                        .table_name
                        .schema()
                        .unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string());

                    let foreign_full_name = if foreign_table.schema().is_some() {
                        foreign_table.value()
                    } else {
                        format!("{}.{}", referencing_schema, foreign_table.value())
                    };

                    let referencing_full_name = if stmt.table_name.schema().is_some() {
                        stmt.table_name.value()
                    } else {
                        format!("{}.{}", referencing_schema, stmt.table_name.value())
                    };

                    let is_self_referencing =
                        foreign_full_name.eq_ignore_ascii_case(&referencing_full_name);

                    let mut ref_schema = if is_self_referencing {
                        schema.clone()
                    } else {
                        self.engine.get_table_schema(&foreign_full_name)?
                    };

                    // Find referenced column name
                    let ref_col_name = if let Some(ref col) = foreign_column {
                        col.value.clone()
                    } else {
                        ref_schema
                            .primary_key_columns()
                            .first()
                            .map(|c| c.name.clone())
                            .ok_or_else(|| {
                                Error::internal(format!(
                                    "referenced table '{}' must have a primary key for unqualified references",
                                    foreign_full_name
                                ))
                            })?
                    };

                    let ref_col = ref_schema
                        .get_column_by_name(&ref_col_name)
                        .ok_or_else(|| Error::column_not_found_by_name(ref_col_name.clone()))?;

                    if ref_col.data_type != schema.columns[col_idx].data_type {
                        return Err(Error::Type(format!(
                            "foreign key type mismatch: {} ({:?}) vs {} ({:?})",
                            col_name,
                            schema.columns[col_idx].data_type,
                            ref_col_name,
                            ref_col.data_type
                        )));
                    }

                    let fk_meta = crate::core::schema::ForeignKeyMetadata {
                        column_id: col_idx,
                        referenced_table: foreign_full_name.clone(),
                        referenced_column_name: ref_col_name,
                        on_delete: ReferentialAction::NoAction,
                        on_update: ReferentialAction::NoAction,
                    };
                    schema.foreign_keys.push(fk_meta);

                    if is_self_referencing {
                        schema.referenced_by.push(referencing_full_name.clone());
                    } else {
                        ref_schema.referenced_by.push(referencing_full_name.clone());
                        schemas_to_update.push(ref_schema);
                    }
                }
            }
        }

        for constraint in &stmt.table_constraints {
            match constraint {
                TableConstraint::Unique(cols) => {
                    let col_names: Vec<String> = cols.iter().map(|c| c.value.clone()).collect();
                    table_unique_constraints.push(col_names);
                }
                TableConstraint::ForeignKey {
                    column,
                    foreign_table,
                    foreign_column,
                    on_delete,
                    on_update,
                    ..
                } => {
                    // Validate that the referencing column exists in our new schema
                    let (col_idx, _) = schema
                        .find_column(&column.value)
                        .ok_or_else(|| Error::column_not_found_by_name(column.value.clone()))?;

                    let referencing_schema = stmt
                        .table_name
                        .schema()
                        .unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string());

                    let foreign_full_name = if foreign_table.schema().is_some() {
                        foreign_table.value()
                    } else {
                        format!("{}.{}", referencing_schema, foreign_table.value())
                    };

                    let referencing_full_name = if stmt.table_name.schema().is_some() {
                        stmt.table_name.value()
                    } else {
                        format!("{}.{}", referencing_schema, stmt.table_name.value())
                    };

                    let is_self_referencing =
                        foreign_full_name.eq_ignore_ascii_case(&referencing_full_name);

                    // Validate that the referenced table exists and get its schema
                    let mut ref_schema = if is_self_referencing {
                        schema.clone()
                    } else {
                        self.engine.get_table_schema(&foreign_full_name)?
                    };

                    // Validate that the referenced column exists and is PK or Unique
                    let ref_col = ref_schema
                        .get_column_by_name(&foreign_column.value)
                        .ok_or_else(|| {
                            Error::column_not_found_by_name(foreign_column.value.clone())
                        })?;

                    // MVP constraint: Must be primary key or unique
                    if !ref_col.primary_key {
                        // Let's just do a loose check for MVP or assume it's enforced elsewhere
                    }

                    // Check type compatibility
                    if ref_col.data_type != schema.columns[col_idx].data_type {
                        return Err(Error::Type(format!(
                            "foreign key type mismatch: {} ({:?}) vs {} ({:?})",
                            column.value,
                            schema.columns[col_idx].data_type,
                            foreign_column.value,
                            ref_col.data_type
                        )));
                    }

                    // Build metadata
                    let fk_meta = crate::core::schema::ForeignKeyMetadata {
                        column_id: col_idx,
                        referenced_table: foreign_full_name.clone(),
                        referenced_column_name: foreign_column.value.clone(),
                        on_delete: *on_delete,
                        on_update: *on_update,
                    };
                    schema.foreign_keys.push(fk_meta);

                    if is_self_referencing {
                        schema.referenced_by.push(referencing_full_name.clone());
                    } else {
                        // Update referenced schema's referenced_by list
                        ref_schema.referenced_by.push(referencing_full_name.clone());
                        schemas_to_update.push(ref_schema);
                    }
                }
                _ => {} // PrimaryKey and Check not fully supported at table level yet
            }
        }

        // Check if there's an active transaction
        let mut active_tx = self.active_transaction.lock().unwrap();

        if let Some(ref mut _tx_state) = *active_tx {
            // Transactional DDL: Create table immediately but log for undo
            tracing::info!(
                "Executing CREATE TABLE for '{}' (in transaction)",
                table_name
            );
            self.engine.create_table(schema.clone())?;

            // Update referenced schemas
            for ref_schema in schemas_to_update {
                self.engine.update_table_schema(
                    &format!("{}.{}", ref_schema.schema_name, ref_schema.table_name),
                    ref_schema,
                )?;
            }

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
            tracing::info!("Executing CREATE TABLE for '{}'", table_name);
            self.engine.create_table(schema)?;

            // Update referenced schemas
            for ref_schema in schemas_to_update {
                self.engine.update_table_schema(
                    &format!("{}.{}", ref_schema.schema_name, ref_schema.table_name),
                    ref_schema,
                )?;
            }
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
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let table_name = &stmt.table_name.value();

        // Prevent dropping tables in reserved namespaces unless internal
        if Schema::is_reserved_namespace(table_name) && !ctx.is_internal() {
            return Err(Error::ReservedNamespaceModification(table_name.clone()));
        }

        // Check if table exists
        if !self.engine.table_exists(table_name)? {
            if stmt.if_exists {
                return Ok(Box::new(ExecResult::empty()));
            }
            return Err(Error::TableNotFoundByName(table_name.clone()));
        }

        // Drop triggers BEFORE acquiring active_tx lock to avoid deadlock with start_transaction_for_dml
        if let Ok(true) = self
            .engine
            .table_exists(crate::storage::triggers::SYS_TRIGGERS)
        {
            let _ = self.delete_table_triggers(table_name);
        }

        // Check if there's an active transaction
        let mut active_tx = self.active_transaction.lock().unwrap();

        if let Some(ref mut _tx_state) = *active_tx {
            tracing::info!("Executing DROP TABLE for '{}' (in transaction)", table_name);
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
            tracing::info!("Executing DROP TABLE for '{}'", table_name);
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
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let table_name = &stmt.table_name.value();

        // Prevent altering tables in reserved namespaces unless internal
        if Schema::is_reserved_namespace(table_name) && !ctx.is_internal() {
            return Err(Error::ReservedNamespaceModification(table_name.clone()));
        }

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

                    // Force a global schema update so subsequent statements in the same session
                    // can see the new column.
                    let schema = table.schema().clone();
                    self.engine.update_table_schema(table_name, schema)?;

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

                    // Force a global schema update
                    let schema = table.schema().clone();
                    self.engine.update_table_schema(table_name, schema)?;

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

                    // Force a global schema update
                    let schema = table.schema().clone();
                    self.engine.update_table_schema(table_name, schema)?;

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

                    let auto_increment = col_def
                        .constraints
                        .iter()
                        .any(|c| matches!(c, ColumnConstraint::AutoIncrement));

                    if auto_increment && data_type != crate::core::DataType::Integer {
                        return Err(Error::InvalidArgumentMessage(
                            "AUTOINCREMENT is only allowed on INTEGER columns".to_string(),
                        ));
                    }

                    let auto_increment_opt = if auto_increment { Some(true) } else { None };

                    let check_expr = col_def.constraints.iter().find_map(|c| {
                        if let ColumnConstraint::Check(expr) = c {
                            Some(expr.to_string())
                        } else {
                            None
                        }
                    });

                    let check_expr_opt = check_expr.map(Some);

                    self.engine.modify_column(
                        table_name,
                        &col_def.name.value,
                        data_type,
                        nullable,
                        auto_increment_opt,
                        check_expr_opt.clone(),
                    )?;

                    let is_unique = col_def
                        .constraints
                        .iter()
                        .any(|c| matches!(c, ColumnConstraint::Unique));

                    if is_unique {
                        let index_name = format!("unique_{}_{}", table_name, col_def.name.value);
                        table.create_index_with_type(
                            &index_name,
                            &[&col_def.name.value],
                            true, // unique
                            None,
                        )?;
                    }

                    // Record ALTER TABLE MODIFY COLUMN to WAL for persistence
                    self.engine.record_alter_table_modify_column(
                        table_name,
                        &col_def.name.value,
                        data_type,
                        nullable,
                        auto_increment_opt,
                        check_expr_opt,
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
            AlterTableOperation::AddConstraint => {
                if let Some(ref constraint) = stmt.constraint {
                    match constraint {
                        TableConstraint::ForeignKey {
                            column,
                            foreign_table,
                            foreign_column,
                            on_delete,
                            on_update,
                            ..
                        } => {
                            let mut schema = self.engine.get_table_schema(table_name)?;

                            // 1. Validate referencing column exists in our table
                            let (col_idx, _) =
                                schema.find_column(&column.value).ok_or_else(|| {
                                    Error::column_not_found_by_name(column.value.clone())
                                })?;

                            let referencing_schema =
                                ctx.current_schema().unwrap_or("public").to_string();

                            let foreign_full_name = if foreign_table.schema().is_some() {
                                foreign_table.value()
                            } else {
                                format!("{}.{}", referencing_schema, foreign_table.value())
                            };

                            let referencing_full_name =
                                format!("{}.{}", referencing_schema, stmt.table_name.value);

                            // 2. Validate referenced table and column
                            let mut ref_schema =
                                self.engine.get_table_schema(&foreign_full_name)?;
                            let ref_col = ref_schema
                                .get_column_by_name(&foreign_column.value)
                                .ok_or_else(|| {
                                    Error::column_not_found_by_name(foreign_column.value.clone())
                                })?;

                            if ref_col.data_type != schema.columns[col_idx].data_type {
                                return Err(Error::Type(format!(
                                    "foreign key type mismatch: {} ({:?}) vs {} ({:?})",
                                    column.value,
                                    schema.columns[col_idx].data_type,
                                    foreign_column.value,
                                    ref_col.data_type
                                )));
                            }

                            // 3. Validate existing data: make sure there are no orphans
                            let col_indices = vec![col_idx];
                            let mut scanner = table.scan(&col_indices, None)?;
                            while scanner.next() {
                                let row = scanner.row();
                                if let Some(val) = row.get(0) {
                                    if val.is_null() {
                                        continue;
                                    }

                                    // Query the referenced table
                                    let ref_table_name = foreign_full_name.to_lowercase();
                                    let ref_table = tx.get_table(&ref_table_name)?;

                                    let mut check_expr =
                                        crate::storage::expression::ComparisonExpr::new(
                                            foreign_column.value.clone(),
                                            crate::core::Operator::Eq,
                                            val.clone(),
                                        );
                                    check_expr.prepare_for_schema(&ref_schema);

                                    let ref_col_indices = vec![0]; // just need to check existence
                                    let mut ref_scanner =
                                        ref_table.scan(&ref_col_indices, Some(&check_expr))?;
                                    if !ref_scanner.next() {
                                        return Err(Error::ReferentialIntegrityViolation {
                                            message: format!(
                                                "ALTER TABLE ADD CONSTRAINT failed: row contains value '{}' not present in {}({})",
                                                val, foreign_full_name, foreign_column.value
                                            ),
                                        });
                                    }
                                }
                            }
                            drop(scanner);

                            // 4. Update current table schema
                            let fk_meta = crate::core::schema::ForeignKeyMetadata {
                                column_id: col_idx,
                                referenced_table: foreign_full_name.clone(),
                                referenced_column_name: foreign_column.value.clone(),
                                on_delete: *on_delete,
                                on_update: *on_update,
                            };
                            schema.foreign_keys.push(fk_meta);
                            self.engine.update_table_schema(table_name, schema)?;

                            // 5. Update referenced table schema
                            ref_schema.referenced_by.push(referencing_full_name.clone());
                            self.engine
                                .update_table_schema(&foreign_full_name, ref_schema)?;
                        }
                        _ => {
                            return Err(Error::NotSupportedMessage(
                                "Only ADD CONSTRAINT FOREIGN KEY is supported".to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(Error::InvalidArgumentMessage(
                        "ADD CONSTRAINT requires a constraint definition".to_string(),
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
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        // Resolve schema name
        let schema_name = stmt
            .view_name
            .schema()
            .unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string())
            .to_lowercase();

        // Check if schema exists for qualified view names
        let schemas = self.engine.schemas.read().unwrap();
        if !schemas.contains_key(&schema_name) {
            return Err(Error::SchemaNotFound(schema_name));
        }
        drop(schemas);

        let view_name = &stmt.view_name.table();

        // Check if a table with the same name exists
        if self.engine.table_exists(view_name)? {
            return Err(Error::TableAlreadyExists);
        }

        // Convert the query to SQL string
        let query_sql = stmt.query.to_string();

        // Create the view (engine handles if_not_exists logic)
        self.engine
            .create_view(&schema_name, view_name, query_sql, stmt.if_not_exists)?;

        Ok(Box::new(ExecResult::empty()))
    }

    /// Execute a DROP VIEW statement
    pub(crate) fn execute_drop_view(
        &self,
        stmt: &DropViewStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let schema_name = ctx.current_schema().unwrap_or("public").to_lowercase();
        let view_name = &stmt.view_name.value;

        // Drop the view (engine handles if_exists logic)
        self.engine
            .drop_view(&schema_name, view_name, stmt.if_exists)?;

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

    pub(crate) fn ensure_procedures_table_exists(&self) -> Result<()> {
        let tx = self.engine.begin_transaction()?;
        let tables = tx.list_tables()?;
        let has_procedures_table = tables
            .iter()
            .any(|t| t.eq_ignore_ascii_case(SYS_PROCEDURES));
        drop(tx);

        if !has_procedures_table {
            if let Err(e) = self.execute_internal_sql(CREATE_PROCEDURES_SQL) {
                tracing::error!("Failed to create procedures table: {}", e);
            }
        }

        Ok(())
    }

    fn procedure_exists(&self, procedure_name: &str) -> Result<bool> {
        let tx = self.engine.begin_transaction()?;
        let table = match tx.get_table(SYS_PROCEDURES) {
            Ok(table) => table,
            Err(_) => return Ok(false),
        };

        let mut scanner = table.scan(&[], None)?;
        while scanner.next() {
            let row = scanner.row();
            if let Some(crate::core::value::Value::Text(name)) = row.get(2) {
                if name.eq_ignore_ascii_case(procedure_name) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn insert_procedure(
        &self,
        procedure: &crate::storage::procedures::StoredProcedure,
    ) -> Result<()> {
        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(SYS_PROCEDURES)?;

        let parameters_json = serde_json::to_string(&procedure.parameters).map_err(|e| {
            Error::internal(format!("Failed to serialize procedure parameters: {}", e))
        })?;

        let schema_value = match &procedure.schema {
            Some(schema) => crate::core::value::Value::text(schema.clone()),
            None => crate::core::value::Value::Null(crate::core::types::DataType::Text),
        };

        let row = crate::core::row::Row::from_values(vec![
            crate::core::value::Value::Null(crate::core::types::DataType::Integer),
            schema_value,
            crate::core::value::Value::text(procedure.name.clone()),
            crate::core::value::Value::text(parameters_json),
            crate::core::value::Value::text(procedure.language.clone()),
            crate::core::value::Value::text(procedure.code.clone()),
        ]);

        table.insert(row)?;
        tx.commit()?;
        Ok(())
    }

    fn update_procedure(
        &self,
        procedure: &crate::storage::procedures::StoredProcedure,
    ) -> Result<()> {
        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(SYS_PROCEDURES)?;

        let mut function_id: Option<crate::core::value::Value> = None;

        let mut scanner = table.scan(&[], None)?;
        while scanner.next() {
            let row = scanner.row();
            if let Some(crate::core::value::Value::Text(name)) = row.get(2) {
                if name.eq_ignore_ascii_case(&procedure.name) {
                    function_id = row.get(0).cloned();
                    break;
                }
            }
        }

        if let Some(id_value) = &function_id {
            use crate::storage::expression::{ComparisonExpr, Expression as StorageExpr};
            let mut id_expr =
                ComparisonExpr::new("id", crate::core::Operator::Eq, id_value.clone());
            let schema = table.schema();
            id_expr.prepare_for_schema(schema);
            table.delete(Some(&id_expr))?;
        }

        let parameters_json = serde_json::to_string(&procedure.parameters).map_err(|e| {
            Error::internal(format!("Failed to serialize procedure parameters: {}", e))
        })?;

        let schema_value = match &procedure.schema {
            Some(schema) => crate::core::value::Value::text(schema.clone()),
            None => crate::core::value::Value::Null(crate::core::types::DataType::Text),
        };

        let id_value = function_id.unwrap_or(crate::core::value::Value::Null(
            crate::core::types::DataType::Integer,
        ));

        let new_row = crate::core::row::Row::from_values(vec![
            id_value,
            schema_value,
            crate::core::value::Value::text(procedure.name.clone()),
            crate::core::value::Value::text(parameters_json),
            crate::core::value::Value::text(procedure.language.clone()),
            crate::core::value::Value::text(procedure.code.clone()),
        ]);

        table.insert(new_row)?;
        tx.commit()?;
        Ok(())
    }

    /// Delete a procedure from the system table
    fn delete_procedure(&self, procedure_name: &str) -> Result<()> {
        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(SYS_PROCEDURES)?;

        // Find the procedure ID by name
        let mut scanner = table.scan(&[], None)?;
        let mut procedure_id: Option<crate::core::value::Value> = None;

        while scanner.next() {
            let row = scanner.row();
            if let Some(crate::core::value::Value::Text(name)) = row.get(2) {
                if name.eq_ignore_ascii_case(procedure_name) {
                    procedure_id = row.get(0).cloned(); // ID is at index 0
                    break;
                }
            }
        }

        if let Some(id_value) = procedure_id {
            // Delete by ID using WHERE expression
            use crate::storage::expression::{ComparisonExpr, Expression as StorageExpr};
            let mut id_expr = ComparisonExpr::new("id", crate::core::Operator::Eq, id_value);
            let schema = table.schema();
            id_expr.prepare_for_schema(schema);
            table.delete(Some(&id_expr))?;
        }

        tx.commit()?;
        Ok(())
    }

    pub(crate) fn execute_create_procedure(
        &self,
        stmt: &crate::parser::ast::CreateProcedureStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        self.ensure_procedures_table_exists()?;

        let procedure_name_upper = stmt.procedure_name.function().to_uppercase();
        let exists = self.procedure_exists(&procedure_name_upper)?;

        if exists && !stmt.or_replace {
            return Err(Error::FunctionAlreadyExists(procedure_name_upper.clone()));
        }

        let is_sql = stmt.language.eq_ignore_ascii_case("sql")
            || stmt.language.eq_ignore_ascii_case("plsql")
            || stmt.language.eq_ignore_ascii_case("pl/sql");
        if !is_sql && !self.function_registry.is_language_supported(&stmt.language) {
            return Err(Error::internal(format!(
                "Unsupported language: {}",
                stmt.language
            )));
        }

        let stored_parameters: Vec<crate::storage::procedures::StoredProcedureParameter> = stmt
            .parameters
            .iter()
            .map(|p| crate::storage::procedures::StoredProcedureParameter {
                mode: p.mode.to_string(),
                name: p.name.value.clone(),
                data_type: p.data_type.clone(),
            })
            .collect();

        let stored_procedure = crate::storage::procedures::StoredProcedure {
            id: 0,
            schema: Some(
                stmt.procedure_name
                    .schema()
                    .unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string())
                    .to_uppercase(),
            ),
            name: procedure_name_upper.clone(),
            parameters: stored_parameters,
            language: stmt.language.clone(),
            code: stmt.body.clone(),
        };

        if exists {
            self.update_procedure(&stored_procedure)?;
        } else {
            self.insert_procedure(&stored_procedure)?;
        }

        self.function_registry
            .register_procedure(&procedure_name_upper, stored_procedure);

        Ok(Box::new(EmptyResult::new()))
    }

    /// Execute a DROP PROCEDURE statement
    pub(crate) fn execute_drop_procedure(
        &self,
        stmt: &crate::parser::ast::DropProcedureStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let procedure_name = stmt.procedure_name.function();
        let procedure_name_upper = procedure_name.to_uppercase();

        // Check if procedure exists
        if !self.procedure_exists(&procedure_name_upper)? {
            if stmt.if_exists {
                return Ok(Box::new(EmptyResult::new()));
            }
            return Err(Error::FunctionNotFound(procedure_name.clone()));
        }

        // Delete procedure from system table
        self.delete_procedure(&procedure_name_upper)?;

        // Unregister the procedure from the registry
        self.function_registry
            .unregister_procedure(&procedure_name_upper);

        Ok(Box::new(EmptyResult::new()))
    }

    /// Execute a CREATE FUNCTION statement
    pub(crate) fn execute_create_function(
        &self,
        stmt: &CreateFunctionStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        // Ensure the functions system table exists
        self.ensure_functions_table_exists()?;

        // Check if function already exists
        let function_name_upper = stmt.function_name.function().to_uppercase();
        if self.function_exists(&function_name_upper)? {
            if stmt.if_not_exists {
                return Ok(Box::new(EmptyResult::new()));
            }
            return Err(Error::FunctionAlreadyExists(function_name_upper.clone()));
        }

        // Convert parameters to simplified format
        let stored_parameters: Vec<StoredParameter> = stmt
            .parameters
            .iter()
            .map(|p| StoredParameter {
                name: p.name.value.clone(),
                data_type: p.data_type.clone(),
            })
            .collect();

        // Create stored function record
        let stored_function = StoredFunction {
            id: 0, // Will be set by database
            schema: Some(
                stmt.function_name
                    .schema()
                    .unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string())
                    .to_uppercase(),
            ),
            name: function_name_upper.clone(),
            parameters: stored_parameters,
            return_type: stmt.return_type.clone(),
            language: stmt.language.clone(),
            code: stmt.body.clone(),
        };

        // Collect parameter names
        let param_names: Vec<String> = stmt
            .parameters
            .iter()
            .map(|p| p.name.value.clone())
            .collect();

        // Check if the backend exists for this language
        if !self.function_registry.is_language_supported(&stmt.language) {
            return Err(Error::internal(format!(
                "Unsupported language: {}",
                stmt.language
            )));
        }

        // Register the function in the registry first
        self.function_registry.register_user_defined(
            function_name_upper.clone(),
            stmt.body.clone(),
            stmt.language.clone(),
            param_names,
            FunctionSignature::new(
                // TODO: Map return_type string to FunctionDataType
                FunctionDataType::Unknown,
                // TODO: Map parameters to FunctionDataType
                vec![],
                stmt.parameters.len(),
                stmt.parameters.len(),
            ),
        )?;

        // Insert function into system table
        // If this fails, we need to unregister from the registry to maintain consistency
        if let Err(e) = self.insert_function(&stored_function) {
            // Rollback registry registration on database insert failure
            let _ = self
                .function_registry
                .unregister_user_defined(&function_name_upper);
            return Err(e);
        }

        Ok(Box::new(EmptyResult::new()))
    }

    /// Execute a DROP FUNCTION statement
    pub(crate) fn execute_drop_function(
        &self,
        stmt: &DropFunctionStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        let function_name = stmt.function_name.function();

        // Check if function exists
        if !self.function_exists(&function_name)? {
            if stmt.if_exists {
                return Ok(Box::new(EmptyResult::new()));
            }
            return Err(Error::FunctionNotFound(function_name.clone()));
        }

        // Delete function from system table
        self.delete_function(&function_name)?;

        // Unregister the function from the registry
        self.function_registry
            .unregister_user_defined(&function_name)?;

        Ok(Box::new(EmptyResult::new()))
    }

    /// Ensure the functions system table exists
    pub(crate) fn ensure_functions_table_exists(&self) -> Result<()> {
        let tx = self.engine.begin_transaction()?;
        let tables = tx.list_tables()?;
        let has_functions = tables.iter().any(|t| t.eq_ignore_ascii_case(SYS_FUNCTIONS));
        drop(tx);

        if !has_functions {
            if let Err(e) = self.execute_internal_sql(CREATE_FUNCTIONS_SQL) {
                tracing::error!("Failed to create functions table: {}", e);
            }
        }

        Ok(())
    }

    /// Check if a function exists in the system table
    fn function_exists(&self, function_name: &str) -> Result<bool> {
        let tx = self.engine.begin_transaction()?;
        let table = match tx.get_table(SYS_FUNCTIONS) {
            Ok(table) => table,
            Err(_) => return Ok(false), // Table doesn't exist, so function doesn't exist
        };

        // Query for function by name (name is now at index 2, schema at index 1)
        let mut scanner = table.scan(&[], None)?;
        while scanner.next() {
            let row = scanner.row();
            if let Some(Value::Text(name)) = row.get(2) {
                if name.eq_ignore_ascii_case(function_name) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Insert a function into the system table
    fn insert_function(&self, function: &StoredFunction) -> Result<()> {
        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(SYS_FUNCTIONS)?;

        // Serialize parameters to JSON
        let parameters_json = serde_json::to_string(&function.parameters).map_err(|e| {
            Error::internal(format!("Failed to serialize function parameters: {}", e))
        })?;

        // Create row with values in schema order (id is auto-increment, set to NULL)
        let schema_value = match &function.schema {
            Some(schema) => Value::Text(Arc::from(schema.clone())),
            None => Value::Null(DataType::Text),
        };

        let values = vec![
            Value::Null(DataType::Integer),                // id (auto-increment)
            schema_value,                                  // schema
            Value::Text(Arc::from(function.name.clone())), // name
            Value::Text(Arc::from(parameters_json)),       // parameters
            Value::Text(Arc::from(function.return_type.clone())), // return_type
            Value::Text(Arc::from(function.language.clone())), // language
            Value::Text(Arc::from(function.code.clone())), // code
        ];

        let row = Row::from_values(values);
        table.insert(row)?;
        tx.commit()?;

        Ok(())
    }

    /// Delete a function from the system table
    fn delete_function(&self, function_name: &str) -> Result<()> {
        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(SYS_FUNCTIONS)?;

        // Find the function ID by name
        let mut scanner = table.scan(&[], None)?;
        let mut function_id: Option<Value> = None;

        while scanner.next() {
            let row = scanner.row();
            if let Some(Value::Text(name)) = row.get(2) {
                if name.eq_ignore_ascii_case(function_name) {
                    function_id = row.get(0).cloned(); // ID is at index 0
                    break;
                }
            }
        }

        if let Some(id_value) = function_id {
            // Delete by ID using WHERE expression
            use crate::storage::expression::{ComparisonExpr, Expression as StorageExpr};
            let mut id_expr = ComparisonExpr::new("id", crate::core::Operator::Eq, id_value);
            let schema = table.schema();
            id_expr.prepare_for_schema(schema);
            table.delete(Some(&id_expr))?;
        }

        tx.commit()?;
        Ok(())
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

    /// Execute a CREATE SEQUENCE statement
    pub(crate) fn execute_create_sequence(
        &self,
        stmt: &CreateSequenceStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn crate::storage::traits::QueryResult>> {
        let schema_name = stmt
            .name
            .schema()
            .unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string())
            .to_lowercase();
        let name = stmt.name.table().to_string();

        if self.engine.sequence_exists(&schema_name, &name)? {
            if stmt.if_not_exists {
                return Ok(Box::new(crate::executor::result::ExecResult::new(0, 0)));
            }
            return Err(Error::SequenceAlreadyExists(name));
        }

        let mut options = crate::core::SequenceOptions::default();
        if let Some(v) = stmt.start_with {
            options.start_with = v;
        }
        if let Some(v) = stmt.increment_by {
            options.increment_by = v;
        }
        if let Some(v) = stmt.min_value {
            options.min_value = v;
        }
        if let Some(v) = stmt.max_value {
            options.max_value = v;
        }
        options.cycle = stmt.cycle;

        self.engine.create_sequence(&schema_name, &name, options)?;

        Ok(Box::new(crate::executor::result::ExecResult::new(0, 0)))
    }

    pub(crate) fn execute_alter_sequence(
        &self,
        stmt: &AlterSequenceStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn crate::storage::traits::QueryResult>> {
        let schema_name = stmt
            .name
            .schema()
            .unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string())
            .to_lowercase();
        let name = stmt.name.table().to_string();

        if !self.engine.sequence_exists(&schema_name, &name)? {
            if stmt.if_exists {
                return Ok(Box::new(crate::executor::result::ExecResult::new(0, 0)));
            }
            return Err(Error::SequenceNotFound(name));
        }

        // Just recreating with new values conceptually for ALTER, but we might want to preserve the old start_with
        // We'll extract current options, modify them, and alter.
        // For now, this is a basic implementation that just parses the options.
        // Ideally, Engine should expose get_sequence_options

        let mut options = crate::core::SequenceOptions::default();
        if let Some(v) = stmt.restart_with {
            options.start_with = v;
        }
        if let Some(v) = stmt.increment_by {
            options.increment_by = v;
        }
        if let Some(v) = stmt.min_value {
            options.min_value = v;
        }
        if let Some(v) = stmt.max_value {
            options.max_value = v;
        }
        if let Some(v) = stmt.cycle {
            options.cycle = v;
        }

        self.engine.alter_sequence(&schema_name, &name, options)?;

        Ok(Box::new(crate::executor::result::ExecResult::new(0, 0)))
    }

    pub(crate) fn execute_drop_sequence(
        &self,
        stmt: &DropSequenceStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn crate::storage::traits::QueryResult>> {
        let schema_name = stmt
            .name
            .schema()
            .unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string())
            .to_lowercase();
        let name = stmt.name.table().to_string();

        if !self.engine.sequence_exists(&schema_name, &name)? {
            if stmt.if_exists {
                return Ok(Box::new(crate::executor::result::ExecResult::new(0, 0)));
            }
            return Err(Error::SequenceNotFound(name));
        }

        self.engine.drop_sequence(&schema_name, &name)?;

        Ok(Box::new(crate::executor::result::ExecResult::new(0, 0)))
    }

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

    pub(crate) fn ensure_triggers_table_exists(&self) -> Result<()> {
        let tx = self.engine.begin_transaction()?;
        let tables = tx.list_tables()?;
        let has_triggers_table = tables
            .iter()
            .any(|t| t.eq_ignore_ascii_case(crate::storage::triggers::SYS_TRIGGERS));
        drop(tx);

        if !has_triggers_table {
            if let Err(e) = self.execute_internal_sql(crate::storage::triggers::CREATE_TRIGGERS_SQL)
            {
                tracing::error!("Failed to create triggers table: {}", e);
            }
        }
        Ok(())
    }

    fn trigger_exists(&self, trigger_name: &str) -> Result<bool> {
        let tx = self.engine.begin_transaction()?;
        let tables = tx.list_tables()?;
        if !tables
            .iter()
            .any(|t| t.eq_ignore_ascii_case(crate::storage::triggers::SYS_TRIGGERS))
        {
            return Ok(false);
        }

        let table = tx.get_table(crate::storage::triggers::SYS_TRIGGERS)?;
        let mut scanner = table.scan(&[], None)?;

        while scanner.next() {
            let row = scanner.row();
            if let Some(Value::Text(name)) = row.get(2) {
                if name.eq_ignore_ascii_case(trigger_name) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn insert_trigger(&self, trigger: &crate::storage::triggers::StoredTrigger) -> Result<()> {
        let (tx, mut table, auto_commit) =
            self.start_transaction_for_dml(crate::storage::triggers::SYS_TRIGGERS)?;

        let row_values = vec![
            Value::Null(crate::core::DataType::Integer), // id auto increment
            trigger
                .schema
                .as_ref()
                .map(|s| Value::text(s.clone()))
                .unwrap_or(Value::Null(crate::core::DataType::Null)),
            Value::text(trigger.name.clone()),
            Value::text(trigger.table_name.clone()),
            Value::text(trigger.timing.clone()),
            Value::text(trigger.event.clone()),
            Value::Boolean(trigger.for_each_row),
            Value::text(trigger.language.clone()),
            Value::text(trigger.code.clone()),
        ];

        table.insert(crate::core::Row::from(row_values))?;

        if auto_commit {
            if let Some(mut tx) = tx {
                tx.commit()?;
            }
        }
        Ok(())
    }

    fn delete_table_triggers(&self, table_name: &str) -> Result<()> {
        let (tx, mut table, auto_commit) =
            self.start_transaction_for_dml(crate::storage::triggers::SYS_TRIGGERS)?;
        let mut ids_to_delete = Vec::new();
        let mut scanner = table.scan(&[], None)?;

        while scanner.next() {
            let row = scanner.row();
            if let (Some(Value::Integer(id)), Some(Value::Text(target))) = (row.get(0), row.get(3))
            {
                if target.eq_ignore_ascii_case(table_name) {
                    ids_to_delete.push(*id);
                }
            }
        }

        for id in ids_to_delete {
            let mut pk_expr = crate::storage::expression::ComparisonExpr::new(
                "id",
                crate::core::Operator::Eq,
                Value::Integer(id),
            );
            pk_expr.prepare_for_schema(table.schema());
            let _ = table.delete(Some(&pk_expr))?;
        }

        if auto_commit {
            if let Some(mut tx) = tx {
                tx.commit()?;
            }
        }

        self.trigger_registry.remove_table_triggers(table_name);
        Ok(())
    }
    fn delete_trigger(&self, trigger_name: &str) -> Result<bool> {
        let (tx, mut table, auto_commit) =
            self.start_transaction_for_dml(crate::storage::triggers::SYS_TRIGGERS)?;

        let mut ids_to_delete = Vec::new();
        let mut scanner = table.scan(&[], None)?;
        while scanner.next() {
            let row = scanner.row();
            if let (Some(Value::Integer(id)), Some(Value::Text(name))) = (row.get(0), row.get(2)) {
                if name.eq_ignore_ascii_case(trigger_name) {
                    ids_to_delete.push(*id);
                }
            }
        }

        let mut deleted = false;
        use crate::storage::expression::{ComparisonExpr, Expression as StorageExpr};
        for id in ids_to_delete {
            let mut id_expr =
                ComparisonExpr::new("id", crate::core::Operator::Eq, Value::Integer(id));
            let schema = table.schema();
            id_expr.prepare_for_schema(schema);
            table.delete(Some(&id_expr))?;
            deleted = true;
        }

        if auto_commit {
            if let Some(mut tx) = tx {
                tx.commit()?;
            }
        }
        Ok(deleted)
    }

    pub(crate) fn execute_create_schedule(
        &self,
        stmt: &CreateScheduleStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        self.ensure_cron_tables_exist()?;

        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(crate::storage::jobs::SYS_CRON)?;

        // Ensure schedule is valid
        if let Err(e) = stmt.cron_expr.parse::<cron::Schedule>() {
            return Err(Error::internal(format!("Invalid CRON expression: {}", e)));
        }

        let values = vec![
            crate::core::value::Value::Null(DataType::Integer), // ID (auto increment)
            crate::core::value::Value::text(stmt.name.to_uppercase()),
            crate::core::value::Value::text(stmt.cron_expr.clone()),
            crate::core::value::Value::text(stmt.command.clone()),
            crate::core::value::Value::Boolean(true), // active
        ];

        let row = Row::from_values(values);
        table.insert(row)?;
        tx.commit()?;

        Ok(Box::new(ExecResult::new(1, 0)))
    }

    pub(crate) fn execute_alter_schedule(
        &self,
        stmt: &AlterScheduleStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        use crate::executor::expression::RowFilter;

        self.ensure_cron_tables_exist()?;

        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(crate::storage::jobs::SYS_CRON)?;

        let name_upper = stmt.name.to_uppercase();
        let where_expr =
            crate::parser::ast::Expression::Infix(crate::parser::ast::InfixExpression::new(
                crate::parser::token::Token::new(
                    crate::parser::token::TokenType::Operator,
                    "=",
                    crate::parser::token::Position::default(),
                ),
                Box::new(crate::parser::ast::Expression::Identifier(
                    crate::parser::ast::Identifier {
                        token: crate::parser::token::Token::new(
                            crate::parser::token::TokenType::Identifier,
                            "name",
                            crate::parser::token::Position::default(),
                        ),
                        value: "name".to_string(),
                        value_lower: "name".to_string(),
                    },
                )),
                "=".to_string(),
                Box::new(crate::parser::ast::Expression::StringLiteral(
                    crate::parser::ast::StringLiteral {
                        token: crate::parser::token::Token::new(
                            crate::parser::token::TokenType::String,
                            name_upper.clone(),
                            crate::parser::token::Position::default(),
                        ),
                        value: name_upper.clone(),
                        type_hint: None,
                    },
                )),
            ));

        let schema = table.schema();
        let col_names: Vec<String> = schema
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let row_filter = RowFilter::new(&where_expr, &col_names)?;

        let mut scanner = table.scan(&[], None)?;
        let mut id_to_update = None;

        while scanner.next() {
            let row = scanner.row();
            if row_filter.matches(row) {
                if let Some(crate::core::value::Value::Integer(id)) = row.get(0) {
                    id_to_update = Some(*id);
                    break;
                }
            }
        }

        if let Some(id) = id_to_update {
            let pk_expr = crate::storage::expression::ComparisonExpr::new(
                "id".to_string(),
                crate::core::Operator::Eq,
                crate::core::value::Value::Integer(id),
            );

            let mut setter = |mut row: Row| -> Result<(Row, bool)> {
                let _ = row.set(4, crate::core::value::Value::Boolean(stmt.active));
                Ok((row, true))
            };

            table.update(Some(&pk_expr), &mut setter)?;
            tx.commit()?;
            Ok(Box::new(ExecResult::new(0, 1)))
        } else {
            Err(Error::internal(format!(
                "Schedule not found: {}",
                name_upper
            )))
        }
    }

    pub(crate) fn execute_drop_schedule(
        &self,
        stmt: &DropScheduleStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        use crate::executor::expression::RowFilter;

        self.ensure_cron_tables_exist()?;

        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(crate::storage::jobs::SYS_CRON)?;

        let name_upper = stmt.name.to_uppercase();
        let where_expr =
            crate::parser::ast::Expression::Infix(crate::parser::ast::InfixExpression::new(
                crate::parser::token::Token::new(
                    crate::parser::token::TokenType::Operator,
                    "=",
                    crate::parser::token::Position::default(),
                ),
                Box::new(crate::parser::ast::Expression::Identifier(
                    crate::parser::ast::Identifier {
                        token: crate::parser::token::Token::new(
                            crate::parser::token::TokenType::Identifier,
                            "name",
                            crate::parser::token::Position::default(),
                        ),
                        value: "name".to_string(),
                        value_lower: "name".to_string(),
                    },
                )),
                "=".to_string(),
                Box::new(crate::parser::ast::Expression::StringLiteral(
                    crate::parser::ast::StringLiteral {
                        token: crate::parser::token::Token::new(
                            crate::parser::token::TokenType::String,
                            name_upper.clone(),
                            crate::parser::token::Position::default(),
                        ),
                        value: name_upper.clone(),
                        type_hint: None,
                    },
                )),
            ));

        let schema = table.schema();
        let col_names: Vec<String> = schema
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let row_filter = RowFilter::new(&where_expr, &col_names)?;

        let mut scanner = table.scan(&[], None)?;
        let mut id_to_delete = None;

        while scanner.next() {
            let row = scanner.row();
            if row_filter.matches(row) {
                if let Some(crate::core::value::Value::Integer(id)) = row.get(0) {
                    id_to_delete = Some(*id);
                    break;
                }
            }
        }

        let affected = if let Some(id) = id_to_delete {
            let mut pk_expr = crate::storage::expression::ComparisonExpr::new(
                "id".to_string(),
                crate::core::Operator::Eq,
                crate::core::value::Value::Integer(id),
            );
            pk_expr.prepare_for_schema(schema);
            table.delete(Some(&pk_expr))?
        } else {
            0
        };

        tx.commit()?;

        Ok(Box::new(ExecResult::new(0, affected as i64)))
    }

    pub(crate) fn execute_create_trigger(
        &self,
        stmt: &crate::parser::ast::CreateTriggerStatement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        self.ensure_triggers_table_exists()?;

        let trigger_name_upper = stmt.trigger_name.value.to_uppercase();
        let exists = self.trigger_exists(&trigger_name_upper)?;

        if exists && !stmt.if_not_exists {
            return Err(Error::internal(format!(
                "Trigger {} already exists",
                trigger_name_upper
            )));
        } else if exists && stmt.if_not_exists {
            return Ok(Box::new(ExecResult::new(0, 0)));
        }

        if !self.function_registry.is_language_supported(&stmt.language) {
            return Err(Error::internal(format!(
                "Unsupported language for trigger: {}",
                stmt.language
            )));
        }

        let stored_trigger = crate::storage::triggers::StoredTrigger {
            id: 0,
            schema: Some(ctx.current_schema().unwrap_or("public").to_uppercase()),
            name: trigger_name_upper.clone(),
            table_name: stmt.table_name.value().to_uppercase(),
            timing: stmt.timing.to_string(),
            event: stmt.event.to_string(),
            for_each_row: stmt.for_each_row,
            language: stmt.language.clone(),
            code: stmt.body.clone(),
        };

        self.insert_trigger(&stored_trigger)?;
        self.trigger_registry.add_trigger(stored_trigger);

        Ok(Box::new(ExecResult::new(1, 0)))
    }

    pub(crate) fn execute_drop_trigger(
        &self,
        stmt: &crate::parser::ast::DropTriggerStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        self.ensure_triggers_table_exists()?;

        let trigger_name_upper = stmt.trigger_name.value.to_uppercase();
        let deleted = self.delete_trigger(&trigger_name_upper)?;

        if deleted {
            self.trigger_registry.remove_trigger(&trigger_name_upper);
        }

        if !deleted && !stmt.if_exists {
            return Err(Error::internal(format!(
                "Trigger {} does not exist",
                trigger_name_upper
            )));
        }

        Ok(Box::new(ExecResult::new(if deleted { 1 } else { 0 }, 0)))
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
