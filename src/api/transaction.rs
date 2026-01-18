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

//! Transaction API
//!
//! Provides ACID transaction support with the same ergonomic API as Database.
//!
//! # Examples
//!
//! ```ignore
//! use oxibase::Database;
//!
//! let db = Database::open("memory://")?;
//! db.execute("CREATE TABLE accounts (id INTEGER, balance INTEGER)", ())?;
//! db.execute("INSERT INTO accounts VALUES ($1, $2), ($3, $4)", (1, 1000, 2, 500))?;
//!
//! // Transfer money atomically
//! let mut tx = db.begin()?;
//! tx.execute("UPDATE accounts SET balance = balance - $1 WHERE id = $2", (100, 1))?;
//! tx.execute("UPDATE accounts SET balance = balance + $1 WHERE id = $2", (100, 2))?;
//! tx.commit()?;
//! ```

use std::sync::{Arc, Mutex};

use crate::core::{Error, Result, Value};
use crate::executor::{ExecutionContext, Executor};
use crate::parser::ast::Statement;
use crate::parser::Parser;
use crate::storage::traits::{QueryResult, Transaction as StorageTransaction};

use super::database::FromValue;
use super::params::Params;
use super::rows::Rows;
use crate::api::database_ops::{DatabaseOps, TransactionDatabase};
use crate::executor::expression::evaluator_bridge::ExpressionEval;

use crate::storage::traits::EmptyResult;

/// Transaction represents a database transaction
///
/// Provides ACID guarantees for a series of database operations.
/// Must be explicitly committed or rolled back.
pub struct Transaction {
    tx: std::sync::Mutex<Option<Box<dyn StorageTransaction>>>,
    executor: std::sync::Arc<std::sync::Mutex<Executor>>,
    committed: bool,
    rolled_back: bool,
    is_procedure_tx: bool,
    nested: bool,
}

impl Transaction {
    /// Create a new transaction wrapper
    pub(crate) fn new(
        tx: Box<dyn StorageTransaction>,
        executor: std::sync::Arc<std::sync::Mutex<Executor>>,
    ) -> Self {
        Self {
            tx: std::sync::Mutex::new(Some(tx)),
            executor,
            committed: false,
            rolled_back: false,
            is_procedure_tx: false,
            nested: false,
        }
    }

    /// Check if the transaction is still active
    fn check_active(&self) -> Result<()> {
        if self.committed {
            return Err(Error::TransactionEnded);
        }
        if self.rolled_back {
            return Err(Error::TransactionEnded);
        }
        // For procedure transactions, tx is always None and they're always active
        if self.is_procedure_tx {
            return Ok(());
        }
        let guard = self.tx.lock().unwrap();
        if guard.is_none() {
            return Err(Error::TransactionNotStarted);
        }
        Ok(())
    }

    /// Get the transaction ID
    pub fn id(&self) -> i64 {
        self.tx
            .lock()
            .unwrap()
            .as_ref()
            .map(|tx| tx.id())
            .unwrap_or(-1)
    }

    /// Get access to the executor (for internal use)
    #[allow(dead_code)]
    pub(crate) fn executor(&self) -> Arc<Mutex<crate::executor::Executor>> {
        Arc::clone(&self.executor)
    }

    /// Execute query directly through this transaction's executor
    /// This is used internally for procedure execution within transactions
    #[allow(dead_code)]
    pub(crate) fn query_sql_direct(&mut self, sql: &str) -> Result<crate::api::rows::Rows> {
        let executor = self
            .executor
            .lock()
            .map_err(|_| Error::LockAcquisitionFailed("executor".to_string()))?;
        let result = executor.execute(sql)?;
        Ok(crate::api::rows::Rows::new(result))
    }

    /// Execute a SQL statement within the transaction
    ///
    /// # Parameters
    ///
    /// Parameters can be passed using:
    /// - Empty tuple `()` for no parameters
    /// - Tuple syntax `(1, "Alice", 30)` for multiple parameters
    /// - `params!` macro `params![1, "Alice", 30]`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut tx = db.begin()?;
    /// tx.execute("INSERT INTO users VALUES ($1, $2)", (1, "Alice"))?;
    /// tx.execute("UPDATE accounts SET balance = balance - $1 WHERE user_id = $2", (100, 1))?;
    /// tx.commit()?;
    /// ```
    pub fn execute<P: Params>(&self, sql: &str, params: P) -> Result<i64> {
        self.check_active()?;

        let param_values = params.into_params();
        let result = self.execute_sql(sql, &param_values)?;
        Ok(result.rows_affected())
    }

    /// Execute a query within the transaction
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut tx = db.begin()?;
    /// for row in tx.query("SELECT * FROM users WHERE age > $1", (18,))? {
    ///     let row = row?;
    ///     println!("{}", row.get::<String>("name")?);
    /// }
    /// tx.commit()?;
    /// ```
    pub fn query<P: Params>(&self, sql: &str, params: P) -> Result<Rows> {
        self.check_active()?;

        let param_values = params.into_params();
        let result = self.execute_sql(sql, &param_values)?;
        Ok(Rows::new(result))
    }

    /// Execute a query and return a single value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut tx = db.begin()?;
    /// let count: i64 = tx.query_one("SELECT COUNT(*) FROM users", ())?;
    /// tx.commit()?;
    /// ```
    pub fn query_one<T: FromValue, P: Params>(&self, sql: &str, params: P) -> Result<T> {
        let mut rows = self.query(sql, params)?;
        let row = rows.next().ok_or(Error::NoRowsReturned)??;
        row.get(0)
    }

    /// Execute a query and return an optional single value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut tx = db.begin()?;
    /// let name: Option<String> = tx.query_opt("SELECT name FROM users WHERE id = $1", (999,))?;
    /// tx.commit()?;
    /// ```
    pub fn query_opt<T: FromValue, P: Params>(&self, sql: &str, params: P) -> Result<Option<T>> {
        match self.query(sql, params)?.next() {
            Some(row) => Ok(Some(row?.get(0)?)),
            None => Ok(None),
        }
    }

    /// Internal SQL execution
    fn execute_sql(&self, sql: &str, params: &[Value]) -> Result<Box<dyn QueryResult>> {
        let mut guard = self.tx.lock().unwrap();
        let is_procedure_tx = guard.is_none();

        let storage_tx = if is_procedure_tx {
            // For procedure transactions, the executor should already have the active transaction
            None
        } else {
            Some(guard.take().unwrap())
        };

        if let Some(tx) = storage_tx {
            self.executor
                .lock()
                .unwrap()
                .set_active_transaction(Some(tx));
        }

        // Parse the SQL
        let mut parser = Parser::new(sql);
        let program = parser
            .parse_program()
            .map_err(|e| Error::parse(e.to_string()))?;

        // Create execution context with parameters
        let ctx = if params.is_empty() {
            ExecutionContext::new()
        } else {
            ExecutionContext::with_params(params.to_vec())
        };

        // Execute each statement
        let mut last_result: Option<Box<dyn QueryResult>> = None;
        for statement in &program.statements {
            last_result = Some(self.execute_statement(statement, &ctx)?);
        }

        // Restore active transaction
        if !is_procedure_tx {
            let storage_tx = self
                .executor
                .lock()
                .unwrap()
                .take_active_transaction()
                .unwrap();
            *guard = Some(storage_tx);
        }

        last_result.ok_or(Error::NoStatementsToExecute)
    }

    /// Execute a single statement
    fn execute_statement(
        &self,
        statement: &Statement,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        match statement {
            Statement::CallProcedure(stmt) => {
                let (procedure, backend, args, param_names_str) = {
                    let executor = self
                        .executor
                        .lock()
                        .map_err(|_| Error::LockAcquisitionFailed("executor".to_string()))?;
                    let procedure_name = stmt.procedure_name.value();
                    let procedure = executor
                        .procedure_registry()
                        .get(&procedure_name)
                        .ok_or_else(|| Error::ProcedureNotFound(procedure_name.clone()))?;
                    let mut args = Vec::new();
                    for arg in &stmt.arguments {
                        let mut eval = ExpressionEval::compile(arg, &[])?;
                        let value = eval.eval_slice(&[])?;
                        args.push(value);
                    }
                    let fr = executor.function_registry();
                    let br = fr.backend_registry();
                    let backend = br
                        .get_backend(procedure.language())
                        .ok_or_else(|| {
                            Error::internal(format!(
                                "No backend found for language: {}",
                                procedure.language()
                            ))
                        })?
                        .clone();
                    let param_names_str: Vec<String> = procedure.param_names().to_vec();
                    (procedure, backend, args, param_names_str)
                };
                let tx_arc = Arc::new(std::sync::Mutex::new(Transaction {
                    tx: std::sync::Mutex::new(None),
                    executor: Arc::clone(&self.executor),
                    committed: false,
                    rolled_back: false,
                    is_procedure_tx: true,
                    nested: true,
                }));
                let db_ops: Box<dyn DatabaseOps> =
                    Box::new(TransactionDatabase::new(Arc::clone(&tx_arc)));
                let param_refs: Vec<&str> = param_names_str.iter().map(|s| s.as_str()).collect();
                backend.execute_procedure(procedure.code(), &args, &param_refs, db_ops)?;
                Ok(Box::new(EmptyResult::new()))
            }
            _ => self
                .executor
                .lock()
                .unwrap()
                .execute_statement(statement, ctx),
        }
    }

    /// Commit the transaction
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let tx = db.begin()?;
    /// tx.lock().unwrap().execute("INSERT INTO users VALUES ($1, $2)", (1, "Alice"))?;
    /// tx.lock().unwrap().commit()?;
    /// ```
    pub fn commit(&mut self) -> Result<()> {
        if self.committed {
            return Err(Error::TransactionEnded);
        }

        if self.is_procedure_tx {
            if let Some(mut tx) = self.executor.lock().unwrap().take_active_transaction() {
                tx.commit()?;
                self.committed = true;
            }
        } else {
            let mut guard = self.tx.lock().unwrap();
            if guard.is_none() {
                return Err(Error::TransactionNotStarted);
            }

            if let Some(mut tx) = guard.take() {
                tx.commit()?;
                self.committed = true;
            }
        }

        Ok(())
    }

    /// Roll back the transaction
    ///
    /// All changes made within the transaction are discarded.
    pub fn rollback(&mut self) -> Result<()> {
        if self.committed {
            return Err(Error::TransactionCommitted);
        }

        if self.rolled_back {
            return Ok(()); // Already rolled back
        }

        if self.is_procedure_tx {
            if let Some(mut tx) = self.executor.lock().unwrap().take_active_transaction() {
                tx.rollback()?;
                self.rolled_back = true;
            }
        } else {
            let mut guard = self.tx.lock().unwrap();
            if guard.is_none() {
                return Err(Error::TransactionNotStarted);
            }

            if let Some(mut tx) = guard.take() {
                tx.rollback()?;
                self.rolled_back = true;
            }
        }

        Ok(())
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        // Auto-commit for procedure transactions, auto-rollback for regular transactions
        if !self.committed && !self.rolled_back {
            if self.is_procedure_tx && !self.nested {
                let _ = self.commit();
            } else if !self.is_procedure_tx {
                let _ = self.rollback();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::Database;

    #[test]
    fn test_transaction_rollback() {
        let db = Database::open_in_memory().unwrap();
        db.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, value INTEGER)",
            (),
        )
        .unwrap();
        db.execute("INSERT INTO test VALUES ($1, $2)", (1, 100))
            .unwrap();

        let tx = db.begin().unwrap();
        tx.lock()
            .unwrap()
            .execute("UPDATE test SET value = $1 WHERE id = $2", (200, 1))
            .unwrap();
        tx.lock().unwrap().rollback().unwrap();

        let value: i64 = db
            .query_one("SELECT value FROM test WHERE id = $1", (1,))
            .unwrap();
        assert_eq!(value, 100);
    }

    #[test]
    fn test_transaction_query_one() {
        let db = Database::open_in_memory().unwrap();
        db.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, value INTEGER)",
            (),
        )
        .unwrap();
        db.execute("INSERT INTO test VALUES ($1, $2)", (1, 100))
            .unwrap();

        let tx = db.begin().unwrap();
        let value: i64 = tx
            .lock()
            .unwrap()
            .query_one("SELECT value FROM test WHERE id = $1", (1,))
            .unwrap();
        assert_eq!(value, 100);
        tx.lock().unwrap().commit().unwrap();
    }

    #[test]
    fn test_transaction_auto_rollback() {
        let db = Database::open_in_memory().unwrap();
        db.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, value INTEGER)",
            (),
        )
        .unwrap();
        db.execute("INSERT INTO test VALUES ($1, $2)", (1, 100))
            .unwrap();

        {
            let tx = db.begin().unwrap();
            tx.lock()
                .unwrap()
                .execute("UPDATE test SET value = $1 WHERE id = $2", (200, 1))
                .unwrap();
            // tx dropped without commit - should auto-rollback
        }

        let value: i64 = db
            .query_one("SELECT value FROM test WHERE id = $1", (1,))
            .unwrap();
        assert_eq!(value, 100);
    }

    #[test]
    fn test_transaction_query() {
        let db = Database::open_in_memory().unwrap();
        db.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, value INTEGER)",
            (),
        )
        .unwrap();
        db.execute("INSERT INTO test VALUES ($1, $2)", (1, 100))
            .unwrap();

        let tx = db.begin().unwrap();

        // New API: query with params
        for row in tx.lock().unwrap().query("SELECT * FROM test", ()).unwrap() {
            let row = row.unwrap();
            assert_eq!(row.get::<i64>(0).unwrap(), 1);
            assert_eq!(row.get::<i64>(1).unwrap(), 100);
        }

        tx.lock().unwrap().commit().unwrap();
    }

    #[test]
    fn test_committed_transaction_error() {
        let db = Database::open_in_memory().unwrap();
        db.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", ())
            .unwrap();

        let tx = db.begin().unwrap();
        tx.lock().unwrap().commit().unwrap();

        // Should error on further operations
        assert!(tx
            .lock()
            .unwrap()
            .execute("INSERT INTO test VALUES ($1)", (1,))
            .is_err());
        assert!(tx.lock().unwrap().commit().is_err());
    }

    #[test]
    fn test_transaction_id() {
        let db = Database::open_in_memory().unwrap();
        let tx = db.begin().unwrap();
        assert!(tx.lock().unwrap().id() > 0);
    }
}
