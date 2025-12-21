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

//! User-defined functions using Deno runtime
//!
//! This module provides support for user-defined scalar functions written in
//! JavaScript/TypeScript that execute using the Deno runtime.

use std::sync::Arc;
use std::collections::HashMap;
use deno_runtime::deno_core::{v8, serde_v8, JsRuntime, RuntimeOptions};

use crate::core::{Error, Result, Value};
use super::{FunctionInfo, FunctionSignature, ScalarFunction};
use crate::core::types::DataType;

/// User-defined scalar function that executes JavaScript code using Deno
pub struct UserDefinedScalarFunction {
    name: String,
    code: String,
    signature: FunctionSignature,
}

impl UserDefinedScalarFunction {
    /// Create a new user-defined function
    pub fn new(name: impl Into<String>, code: impl Into<String>, signature: FunctionSignature) -> Self {
        Self {
            name: name.into(),
            code: code.into(),
            signature,
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
        // Create a new JavaScript runtime for each function call
        let mut runtime = JsRuntime::new(RuntimeOptions {
            ..Default::default()
        });

        // Convert args to JavaScript values
        let js_args_vec: Vec<serde_json::Value> = args.iter().map(|v| match v {
            Value::Null(_) => serde_json::Value::Null,
            Value::Integer(i) => serde_json::json!(i),
            Value::Float(f) => serde_json::json!(f),
            Value::Text(s) => serde_json::json!(s.as_ref()),
            Value::Boolean(b) => serde_json::json!(b),
            Value::Timestamp(ts) => serde_json::json!(ts.to_rfc3339()),
            Value::Json(j) => serde_json::from_str(j).unwrap_or(serde_json::Value::Null),
        }).collect();

        let js_args = serde_json::to_string(&js_args_vec)
            .map_err(|e| Error::internal(format!("Failed to serialize arguments: {}", e)))?;

        // Execute the function call
        let script = format!("
            globalThis.__user_function = function() {{
                {}
            }};
            globalThis.__user_function.apply(null, {})
        ", self.code, js_args);

        let result = runtime.execute_script("<user_function>", script)
            .map_err(|e| Error::internal(format!("Function execution failed: {}", e)))?;

        // Extract the result
        let scope = &mut runtime.handle_scope();
        let local = v8::Local::new(scope, result);

        // Try to deserialize as JSON first
        match serde_v8::from_v8::<serde_json::Value>(scope, local) {
            Ok(json_value) => match json_value {
                serde_json::Value::String(s) => Ok(Value::Text(Arc::from(s.as_str()))),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Ok(Value::Integer(i))
                    } else if let Some(f) = n.as_f64() {
                        Ok(Value::Float(f))
                    } else {
                        Ok(Value::Float(0.0)) // fallback
                    }
                }
                serde_json::Value::Bool(b) => Ok(Value::Boolean(b)),
                serde_json::Value::Null => Ok(Value::Null(DataType::Null)),
                _ => Ok(Value::Json(Arc::from(json_value.to_string().as_str()))),
            },
            Err(_) => {
                // Fallback: try to convert as string
                if local.is_string() {
                    let string = serde_v8::from_v8::<String>(scope, local)
                        .map_err(|e| Error::internal(format!("Failed to deserialize string result: {}", e)))?;
                    Ok(Value::Text(Arc::from(string.as_str())))
                } else {
                    Err(Error::internal("Failed to deserialize function result"))
                }
            }
        }
}

    fn clone_box(&self) -> Box<dyn ScalarFunction> {
        Box::new(Self {
            name: self.name.clone(),
            code: self.code.clone(),
            signature: self.signature.clone(),
        })
    }
}

/// Registry for user-defined functions
pub struct UserDefinedFunctionRegistry {
    functions: HashMap<String, Arc<UserDefinedScalarFunction>>,
}

impl UserDefinedFunctionRegistry {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Register a user-defined function
    pub fn register(&mut self, name: String, code: String, signature: FunctionSignature) -> Result<()> {
        let udf = Arc::new(UserDefinedScalarFunction::new(name.clone(), code, signature));
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

    /// List all user-defined functions
    pub fn list(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

impl Default for UserDefinedFunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}