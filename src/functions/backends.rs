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

//! Scripting backends for user-defined functions
//!
//! This module provides pluggable scripting backends that allow user-defined
//! functions to be written in different scripting languages.

#[cfg(feature = "js")]
pub mod boa;
pub mod python;
pub mod rhai;
pub mod triggers;

use crate::core::{Result, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for scripting backends
use std::cell::RefCell;

thread_local! {
    /// Thread-local storage for the current SQL runner.
    /// This is safely set during procedure execution to allow scripting engines
    /// to seamlessly call back into the database engine without needing 'static lifetimes.
    pub static CURRENT_SQL_RUNNER: RefCell<Option<*const dyn SqlRunner>> = RefCell::new(None);
}

/// Executes a closure with a thread-local SQL runner available for nested queries
pub fn with_sql_runner<F, R>(runner: Option<&dyn SqlRunner>, f: F) -> R
where
    F: FnOnce() -> R,
{
    // Unsafe casting to extend lifetime to static temporarily
    // This is safe because we clear the thread local before returning,
    // guaranteeing the reference isn't used after it becomes invalid.
    let ptr = runner.map(|r| {
        let r_static: &'static dyn SqlRunner = unsafe { std::mem::transmute(r) };
        r_static as *const dyn SqlRunner
    });
    CURRENT_SQL_RUNNER.with(|r| *r.borrow_mut() = ptr);

    // Execute the closure (which will run the scripting VM)
    let result = f();

    // Cleanup to ensure no dangling pointers
    CURRENT_SQL_RUNNER.with(|r| *r.borrow_mut() = None);

    result
}

/// Executes a native SQL query using the thread-local runner injected by the current execution context
pub fn execute_sql_query(
    sql: &str,
) -> crate::core::Result<Box<dyn crate::storage::traits::QueryResult>> {
    CURRENT_SQL_RUNNER.with(|r| {
        if let Some(ptr) = *r.borrow() {
            // SAFETY: We guarantee that CURRENT_SQL_RUNNER is only set during the execution of a procedure
            // using `with_sql_runner`, and is cleared before the runner reference is dropped.
            let runner = unsafe { &*ptr };
            runner.execute_query(sql)
        } else {
            Err(crate::core::Error::internal(
                "Cannot execute SQL: No database context available in this procedure",
            ))
        }
    })
}

pub fn commit_transaction() -> crate::core::Result<()> {
    CURRENT_SQL_RUNNER.with(|r| {
        if let Some(ptr) = *r.borrow() {
            let runner = unsafe { &*ptr };
            runner.commit()
        } else {
            Err(crate::core::Error::internal(
                "Cannot commit: No database context available in this procedure",
            ))
        }
    })
}

pub fn rollback_transaction() -> crate::core::Result<()> {
    CURRENT_SQL_RUNNER.with(|r| {
        if let Some(ptr) = *r.borrow() {
            let runner = unsafe { &*ptr };
            runner.rollback()
        } else {
            Err(crate::core::Error::internal(
                "Cannot rollback: No database context available in this procedure",
            ))
        }
    })
}

pub fn begin_transaction() -> crate::core::Result<()> {
    CURRENT_SQL_RUNNER.with(|r| {
        if let Some(ptr) = *r.borrow() {
            let runner = unsafe { &*ptr };
            runner.begin()
        } else {
            Err(crate::core::Error::internal(
                "Cannot begin: No database context available in this procedure",
            ))
        }
    })
}

/// A trait to allow scripting backends to execute native SQL queries
pub trait SqlRunner: Send + Sync {
    fn execute_query(&self, sql: &str) -> Result<Box<dyn crate::storage::traits::QueryResult>>;

    fn execute_ast(
        &self,
        stmt: &crate::parser::ast::Statement,
    ) -> Result<Box<dyn crate::storage::traits::QueryResult>>;

    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
    fn begin(&self) -> Result<()>;
}

pub trait ScriptingBackend {
    /// Get the name of this backend
    fn name(&self) -> &'static str;

    /// Get the list of supported language identifiers for this backend
    fn supported_languages(&self) -> &[&'static str];

    /// Execute script code with the given arguments and parameter names
    fn execute(&self, code: &str, args: &[Value], param_names: &[&str]) -> Result<Value>;

    /// Execute a stored procedure and return potentially modified argument values (OUT/INOUT parameters)
    fn execute_procedure(
        &self,
        code: &str,
        args: &mut [Value],
        param_names: &[&str],
        _modes: &[&str],
        _runner: Option<&dyn SqlRunner>,
    ) -> Result<()> {
        // Default implementation falls back to executing normally and ignoring mutations.
        // Backends should override this to capture mutated state.
        self.execute(code, args, param_names).map(|_| ())
    }

    /// Validate that the code is syntactically correct for this backend
    fn validate_code(&self, code: &str) -> Result<()>;
}

/// Create a backend registry with all enabled backends
pub fn create_backend_registry() -> BackendRegistry {
    let mut registry = BackendRegistry::new();

    // Always include Rhai backend
    registry.register_backend(Arc::new(rhai::RhaiBackend::new()));

    // Note: PL/SQL backend needs a FunctionRegistry reference, so it's registered
    // outside of create_backend_registry or created in a two-step initialization.
    // For now we will manually register it in FunctionRegistry::new().

    // Include optional backends based on features
    #[cfg(feature = "js")]
    registry.register_backend(Arc::new(boa::BoaBackend::new()));

    #[cfg(feature = "python")]
    registry.register_backend(Arc::new(python::PythonBackend::new()));

    registry
}

/// Registry for managing scripting backends
pub struct BackendRegistry {
    backends: HashMap<String, Arc<dyn ScriptingBackend + Send + Sync>>,
}

impl BackendRegistry {
    /// Create a new empty backend registry
    pub fn new() -> Self {
        Self {
            backends: HashMap::new(),
        }
    }

    /// Register a scripting backend
    pub fn register_backend(&mut self, backend: Arc<dyn ScriptingBackend + Send + Sync>) {
        for &lang in backend.supported_languages() {
            self.backends.insert(lang.to_lowercase(), backend.clone());
        }
    }

    /// Get a backend for the given language
    pub fn get_backend(&self, language: &str) -> Option<&Arc<dyn ScriptingBackend + Send + Sync>> {
        self.backends.get(&language.to_lowercase())
    }

    /// Check if a language is supported by any backend
    pub fn is_language_supported(&self, language: &str) -> bool {
        self.backends.contains_key(&language.to_lowercase())
    }

    /// Get all supported languages
    pub fn list_supported_languages(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}
