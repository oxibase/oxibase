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

//! Database operations trait for procedures and functions
//!
//! This trait abstracts database operations to allow procedures to participate
//! in transactions properly.

use std::sync::Arc;

use crate::api::rows::Rows;
use crate::core::Result;

/// Trait for database operations that procedures can use
pub trait DatabaseOps: Send + Sync {
    /// Execute a SQL statement and return number of affected rows
    fn execute(&mut self, sql: &str, params: ()) -> Result<i64>;

    /// Execute a query and return rows
    fn query(&mut self, sql: &str, params: ()) -> Result<Rows>;

    /// Clone the database operations interface
    fn clone_box(&self) -> Box<dyn DatabaseOps>;
}

impl Clone for Box<dyn DatabaseOps> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Database wrapper that implements DatabaseOps
pub struct DatabaseWrapper {
    db: Arc<crate::Database>,
}

impl DatabaseWrapper {
    pub fn new(db: Arc<crate::Database>) -> Self {
        Self { db }
    }
}

impl DatabaseOps for DatabaseWrapper {
    fn execute(&mut self, sql: &str, _params: ()) -> Result<i64> {
        self.db.execute(sql, ())
    }

    fn query(&mut self, sql: &str, _params: ()) -> Result<Rows> {
        self.db.query(sql, ())
    }

    fn clone_box(&self) -> Box<dyn DatabaseOps> {
        Box::new(DatabaseWrapper {
            db: Arc::clone(&self.db),
        })
    }
}

/// Transaction database that implements DatabaseOps for transactional operations
pub struct TransactionDatabase {
    tx: Arc<std::sync::Mutex<crate::api::Transaction>>,
}

impl TransactionDatabase {
    pub fn new(tx: Arc<std::sync::Mutex<crate::api::Transaction>>) -> Self {
        Self { tx }
    }
}

impl DatabaseOps for TransactionDatabase {
    fn execute(&mut self, sql: &str, _params: ()) -> Result<i64> {
        // Execute SQL through the transaction's executor, which now has active_transaction set
        let tx = self.tx.lock().unwrap();
        tx.execute(sql, ())
    }

    fn query(&mut self, sql: &str, _params: ()) -> Result<Rows> {
        // Execute query through the transaction
        let tx = self.tx.lock().unwrap();
        tx.query(sql, ())
    }

    fn clone_box(&self) -> Box<dyn DatabaseOps> {
        Box::new(TransactionDatabase {
            tx: Arc::clone(&self.tx),
        })
    }
}
