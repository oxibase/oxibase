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

#[cfg(feature = "boa")]
pub mod boa;
pub mod python;
pub mod rhai;

use crate::core::{Result, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for scripting backends
pub trait ScriptingBackend {
    /// Get the name of this backend
    fn name(&self) -> &'static str;

    /// Get the list of supported language identifiers for this backend
    fn supported_languages(&self) -> &[&'static str];

    /// Execute script code with the given arguments and parameter names
    fn execute(&self, code: &str, args: &[Value], param_names: &[&str]) -> Result<Value>;

    /// Validate that the code is syntactically correct for this backend
    fn validate_code(&self, code: &str) -> Result<()>;
}

/// Create a backend registry with all enabled backends
pub fn create_backend_registry() -> BackendRegistry {
    let mut registry = BackendRegistry::new();

    // Always include Rhai backend
    registry.register_backend(Arc::new(rhai::RhaiBackend::new()));

    // Include optional backends based on features
    #[cfg(feature = "boa")]
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
