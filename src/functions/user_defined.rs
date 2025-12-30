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

//! User-defined functions with pluggable scripting backends
//!
//! This module provides support for user-defined scalar functions written in
//! various scripting languages using pluggable backends.

use std::collections::HashMap;
use std::sync::Arc;

use super::backends::BackendRegistry;
use super::{FunctionInfo, FunctionSignature, ScalarFunction};
use crate::core::{Error, Result, Value};

/// User-defined scalar function with pluggable scripting backends
pub struct UserDefinedScalarFunction {
    name: String,
    code: String,
    language: String,
    signature: FunctionSignature,
    backend_registry: Arc<BackendRegistry>,
}

impl UserDefinedScalarFunction {
    /// Create a new user-defined function
    pub fn new(
        name: impl Into<String>,
        code: impl Into<String>,
        language: impl Into<String>,
        signature: FunctionSignature,
        backend_registry: Arc<BackendRegistry>,
    ) -> Self {
        Self {
            name: name.into(),
            code: code.into(),
            language: language.into(),
            signature,
            backend_registry,
        }
    }
}

impl ScalarFunction for UserDefinedScalarFunction {
    fn name(&self) -> &str {
        &self.name
    }

    fn info(&self) -> FunctionInfo {
        FunctionInfo::new(
            self.name.clone(),
            super::FunctionType::Scalar,
            "User-defined function".to_string(),
            self.signature.clone(),
        )
    }

    fn evaluate(&self, args: &[Value]) -> Result<Value> {
        // Get the appropriate backend for this function's language
        let backend = self.backend_registry.get_backend(&self.language)
            .ok_or_else(|| Error::internal(format!("No backend available for language: {}", self.language)))?;

        // Execute using the backend
        backend.execute(&self.code, args)
    }

    fn clone_box(&self) -> Box<dyn ScalarFunction> {
        Box::new(Self {
            name: self.name.clone(),
            code: self.code.clone(),
            language: self.language.clone(),
            signature: self.signature.clone(),
            backend_registry: self.backend_registry.clone(),
        })
    }
}

/// Registry for user-defined functions
pub struct UserDefinedFunctionRegistry {
    functions: HashMap<String, Arc<UserDefinedScalarFunction>>,
    backend_registry: Arc<BackendRegistry>,
}

impl UserDefinedFunctionRegistry {
    pub fn new(backend_registry: Arc<BackendRegistry>) -> Self {
        Self {
            functions: HashMap::new(),
            backend_registry,
        }
    }

    /// Register a user-defined function
    pub fn register(
        &mut self,
        name: String,
        code: String,
        language: String,
        signature: FunctionSignature,
    ) -> Result<()> {
        // Validate that the backend exists for this language
        if !self.backend_registry.is_language_supported(&language) {
            return Err(Error::internal(format!("Unsupported language: {}", language)));
        }

        let udf = Arc::new(UserDefinedScalarFunction::new(
            name.clone(),
            code,
            language,
            signature,
            self.backend_registry.clone(),
        ));
        self.functions.insert(name.to_uppercase(), udf);
        Ok(())
    }

    /// Get a user-defined function
    pub fn get(&self, name: &str) -> Option<Arc<UserDefinedScalarFunction>> {
        self.functions.get(&name.to_uppercase()).cloned()
    }

    /// Check if a function exists
    pub fn exists(&self, name: &str) -> bool {
        self.functions.contains_key(&name.to_uppercase())
    }

    /// Unregister a user-defined function
    pub fn unregister(&mut self, name: &str) -> Result<()> {
        let key = name.to_uppercase();
        if self.functions.remove(&key).is_none() {
            return Err(Error::FunctionNotFound(name.to_string()));
        }
        Ok(())
    }

    /// List all user-defined functions
    pub fn list(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

impl Default for UserDefinedFunctionRegistry {
    fn default() -> Self {
        // This should not be used directly - backend registry is required
        panic!("UserDefinedFunctionRegistry::default() should not be called directly. Use new() with a backend registry.");
    }
}
