// Copyright 2026 Oxibase Contributors
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

//! Ring Buffer Table implementation for telemetry
//!
//! Provides a non-MVCC, fixed-capacity, memory-bounded table for internal logs,
//! metrics, and traces that bypasses the WAL.

use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

use crate::core::{DataType, Error, Result, Row, Schema, Value};
use crate::storage::expression::Expression;
use crate::storage::traits::{QueryResult, Scanner, Table};

use std::sync::Arc;

/// A memory-bounded ring buffer table intended for telemetry data.
/// It bypasses MVCC and the WAL.
pub struct SystemRingBufferTable {
    name: String,
    schema: Schema,
    capacity: usize,
    buffer: Arc<RwLock<VecDeque<Row>>>,
}

impl SystemRingBufferTable {
    /// Creates a new SystemRingBufferTable
    pub fn new(
        name: impl Into<String>,
        schema: Schema,
        capacity: usize,
        buffer: Arc<RwLock<VecDeque<Row>>>,
    ) -> Self {
        Self {
            name: name.into(),
            schema,
            capacity,
            buffer,
        }
    }
}

impl Table for SystemRingBufferTable {
    fn name(&self) -> &str {
        &self.name
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn create_column(
        &mut self,
        _name: &str,
        _column_type: DataType,
        _nullable: bool,
    ) -> Result<()> {
        Err(Error::NotSupportedMessage(
            "Schema changes are not supported on system ring buffer tables".to_string(),
        ))
    }

    fn drop_column(&mut self, _name: &str) -> Result<()> {
        Err(Error::NotSupportedMessage(
            "Schema changes are not supported on system ring buffer tables".to_string(),
        ))
    }

    fn insert(&mut self, row: Row) -> Result<Row> {
        let mut buf = self.buffer.write();
        if buf.len() >= self.capacity {
            buf.pop_front();
        }
        buf.push_back(row.clone());
        Ok(row)
    }

    fn insert_batch(&mut self, rows: Vec<Row>) -> Result<()> {
        let mut buf = self.buffer.write();
        for row in rows {
            if buf.len() >= self.capacity {
                buf.pop_front();
            }
            buf.push_back(row);
        }
        Ok(())
    }

    fn update(
        &mut self,
        _where_expr: Option<&dyn Expression>,
        _setter: &mut dyn FnMut(Row) -> Result<(Row, bool)>,
    ) -> Result<i32> {
        Err(Error::NotSupportedMessage(
            "Updates are not supported on system ring buffer tables".to_string(),
        ))
    }

    fn delete(&mut self, _where_expr: Option<&dyn Expression>) -> Result<i32> {
        Err(Error::NotSupportedMessage(
            "Deletes are not supported on system ring buffer tables".to_string(),
        ))
    }

    fn collect_rows_with_limit(
        &self,
        where_expr: Option<&dyn Expression>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Row>> {
        let buf = self.buffer.read();
        let rows = if let Some(expr) = where_expr {
            buf.iter()
                .filter(|row| matches!(expr.evaluate(row), Ok(true)))
                .skip(offset)
                .take(limit)
                .cloned()
                .collect()
        } else {
            buf.iter().skip(offset).take(limit).cloned().collect()
        };
        Ok(rows)
    }

    fn collect_rows_with_limit_unordered(
        &self,
        where_expr: Option<&dyn Expression>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Row>> {
        self.collect_rows_with_limit(where_expr, limit, offset)
    }

    fn collect_all_rows(&self, where_expr: Option<&dyn Expression>) -> Result<Vec<Row>> {
        let buf = self.buffer.read();
        let rows = if let Some(expr) = where_expr {
            buf.iter()
                .filter(|row| matches!(expr.evaluate(row), Ok(true)))
                .cloned()
                .collect()
        } else {
            buf.iter().cloned().collect()
        };
        Ok(rows)
    }

    fn scan(
        &self,
        column_indices: &[usize],
        where_expr: Option<&dyn Expression>,
    ) -> Result<Box<dyn Scanner>> {
        let rows = self.collect_all_rows(where_expr)?;

        let projected_rows: Vec<(i64, Row)> = rows
            .into_iter()
            .enumerate()
            .map(|(i, row)| {
                let values: Vec<Value> = column_indices
                    .iter()
                    .map(|&idx| row.get(idx).cloned().unwrap_or(Value::null_unknown()))
                    .collect();
                (i as i64, Row::from_values(values))
            })
            .collect();

        Ok(Box::new(
            crate::storage::mvcc::scanner::MVCCScanner::from_rows(
                projected_rows,
                self.schema.clone(),
                column_indices.to_vec(),
            ),
        ))
    }

    fn row_count(&self) -> usize {
        self.buffer.read().len()
    }

    fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn commit(&mut self) -> Result<()> {
        Ok(())
    }

    fn rollback(&mut self) {}

    fn rollback_to_timestamp(&self, _timestamp: i64) {}

    fn has_local_changes(&self) -> bool {
        false
    }

    fn get_pending_versions(&self) -> Vec<(i64, Row, bool, i64)> {
        Vec::new()
    }

    fn create_index(&self, _name: &str, _columns: &[&str], _is_unique: bool) -> Result<()> {
        Err(Error::NotSupportedMessage(
            "Indexes are not supported on system ring buffer tables".to_string(),
        ))
    }

    fn drop_index(&self, _name: &str) -> Result<()> {
        Err(Error::NotSupportedMessage(
            "Indexes are not supported on system ring buffer tables".to_string(),
        ))
    }

    fn create_btree_index(
        &self,
        _column_name: &str,
        _is_unique: bool,
        _custom_name: Option<&str>,
    ) -> Result<()> {
        Err(Error::NotSupportedMessage(
            "Indexes are not supported on system ring buffer tables".to_string(),
        ))
    }

    fn drop_btree_index(&self, _column_name: &str) -> Result<()> {
        Err(Error::NotSupportedMessage(
            "Indexes are not supported on system ring buffer tables".to_string(),
        ))
    }

    fn rename_column(&self, _old_name: &str, _new_name: &str) -> Result<()> {
        Err(Error::NotSupportedMessage(
            "Schema changes are not supported on system ring buffer tables".to_string(),
        ))
    }

    fn modify_column(
        &self,
        _name: &str,
        _column_type: DataType,
        _nullable: bool,
        _auto_increment: Option<bool>,
        _check_expr: Option<Option<String>>,
    ) -> Result<()> {
        Err(Error::NotSupportedMessage(
            "Schema changes are not supported on system ring buffer tables".to_string(),
        ))
    }

    fn select(
        &self,
        _columns: &[&str],
        _expr: Option<&dyn Expression>,
    ) -> Result<Box<dyn QueryResult>> {
        Err(Error::NotSupportedMessage(
            "select not implemented for SystemRingBufferTable yet".to_string(),
        ))
    }

    fn select_with_aliases(
        &self,
        _columns: &[&str],
        _expr: Option<&dyn Expression>,
        _aliases: &FxHashMap<String, String>,
    ) -> Result<Box<dyn QueryResult>> {
        Err(Error::NotSupportedMessage(
            "select_with_aliases not implemented for SystemRingBufferTable yet".to_string(),
        ))
    }

    fn select_as_of(
        &self,
        _columns: &[&str],
        _expr: Option<&dyn Expression>,
        _temporal_type: &str,
        _temporal_value: i64,
    ) -> Result<Box<dyn QueryResult>> {
        Err(Error::NotSupportedMessage(
            "Time travel is not supported on system ring buffer tables".to_string(),
        ))
    }
}
