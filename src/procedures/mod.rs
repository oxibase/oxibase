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

//! Stored procedures with pluggable scripting backends
//!
//! This module provides support for stored procedures written in
//! various scripting languages using pluggable backends.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::core::{Error, Result};
use crate::functions::backends::BackendRegistry;

/// Stored procedure with pluggable scripting backends
pub struct StoredProcedure {
    name: String,
    code: String,
    language: String,
    param_names: Vec<String>,
    _backend_registry: Arc<BackendRegistry>,
}

impl StoredProcedure {
    /// Create a new stored procedure
    pub fn new(
        name: impl Into<String>,
        code: impl Into<String>,
        language: impl Into<String>,
        param_names: Vec<String>,
        backend_registry: Arc<BackendRegistry>,
    ) -> Self {
        Self {
            name: name.into(),
            code: code.into(),
            language: language.into(),
            param_names,
            _backend_registry: backend_registry,
        }
    }

    /// Get the procedure name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the procedure code
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Get the language
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Get parameter names
    pub fn param_names(&self) -> &[String] {
        &self.param_names
    }
}

/// Registry for managing stored procedures
pub struct ProcedureRegistry {
    procedures: RwLock<HashMap<String, Arc<StoredProcedure>>>,
    backend_registry: Arc<BackendRegistry>,
}

impl ProcedureRegistry {
    pub fn new(backend_registry: Arc<BackendRegistry>) -> Self {
        Self {
            procedures: RwLock::new(HashMap::new()),
            backend_registry,
        }
    }

    /// Register a stored procedure
    pub fn register(
        &self,
        name: String,
        code: String,
        language: String,
        param_names: Vec<String>,
    ) -> Result<()> {
        // Validate that the backend exists for this language
        if !self.backend_registry.is_language_supported(&language) {
            return Err(Error::internal(format!(
                "Unsupported language: {}",
                language
            )));
        }

        let procedure = Arc::new(StoredProcedure::new(
            name.clone(),
            code,
            language,
            param_names,
            self.backend_registry.clone(),
        ));
        self.procedures
            .write()
            .unwrap()
            .insert(name.to_uppercase(), procedure);
        Ok(())
    }

    /// Get a stored procedure
    pub fn get(&self, name: &str) -> Option<Arc<StoredProcedure>> {
        self.procedures
            .read()
            .unwrap()
            .get(&name.to_uppercase())
            .cloned()
    }

    /// Check if a procedure exists
    pub fn exists(&self, name: &str) -> bool {
        self.procedures
            .read()
            .unwrap()
            .contains_key(&name.to_uppercase())
    }

    /// Unregister a stored procedure
    pub fn unregister(&self, name: &str) -> Result<()> {
        let key = name.to_uppercase();
        if self.procedures.write().unwrap().remove(&key).is_none() {
            return Err(Error::ProcedureNotFound(name.to_string()));
        }
        Ok(())
    }

    /// List all stored procedures
    pub fn list(&self) -> Vec<String> {
        self.procedures.read().unwrap().keys().cloned().collect()
    }

    /// Check if a language is supported
    pub fn is_language_supported(&self, language: &str) -> bool {
        self.backend_registry.is_language_supported(language)
    }
}

impl Clone for ProcedureRegistry {
    fn clone(&self) -> Self {
        Self {
            procedures: RwLock::new(self.procedures.read().unwrap().clone()),
            backend_registry: self.backend_registry.clone(),
        }
    }
}

impl Default for ProcedureRegistry {
    fn default() -> Self {
        // This should not be used directly - backend registry is required
        panic!("ProcedureRegistry::default() should not be called directly. Use new() with a backend registry.");
    }
}
