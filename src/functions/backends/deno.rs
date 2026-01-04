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

//! Deno scripting backend for user-defined functions

use super::ScriptingBackend;
use crate::core::{Error, Result, Value};

#[cfg(feature = "boa")]
use deno_runtime::deno_core::{serde_v8, v8, JsRuntime, RuntimeOptions};

/// Deno scripting backend
#[cfg(feature = "boa")]
pub struct DenoBackend {
    // Runtime will be created per execution for isolation
}

#[cfg(feature = "boa")]
impl DenoBackend {
    /// Create a new Deno backend
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DenoBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Create secure runtime options with filtered extensions
#[cfg(feature = "boa")]
fn create_secure_runtime_options() -> RuntimeOptions {
    let mut options = RuntimeOptions::default();
    options.extensions.retain(|ext| {
        let name = &ext.name;
        // Disable high-risk and unnecessary extensions, but keep net for fetch
        !name.contains("deno_kv") &&           // KV store
        !name.contains("deno_fs") &&           // File system
        !name.contains("deno_process") &&      // Process spawning
        !name.contains("deno_ffi") &&          // Foreign functions
        !name.contains("deno_napi") &&         // Native addons
        !name.contains("deno_cron") &&         // Cron jobs
        !name.contains("deno_os") &&           // OS interfaces
        !name.contains("deno_webgpu") &&       // GPU access
        !name.contains("deno_canvas") &&       // Canvas API
        !name.contains("deno_webstorage") &&   // localStorage
        !name.contains("deno_node") // Node.js compatibility
    });
    options
}

#[cfg(feature = "boa")]
impl ScriptingBackend for DenoBackend {
    fn name(&self) -> &'static str {
        "deno"
    }

    fn supported_languages(&self) -> &[&'static str] {
        &["deno", "javascript", "js", "typescript", "ts"]
    }

    fn execute(&self, code: &str, args: &[Value], param_names: &[&str]) -> Result<Value> {
        // Create a new JavaScript runtime for each function call
        let mut runtime = JsRuntime::new(create_secure_runtime_options());

        // Create arguments array for compatibility
        let mut args_array = Vec::new();
        for arg in args {
            let js_value = match arg {
                Value::Null(_) => serde_json::Value::Null,
                Value::Integer(i) => serde_json::json!(i),
                Value::Float(f) => serde_json::json!(f),
                Value::Text(s) => serde_json::json!(s.as_ref()),
                Value::Boolean(b) => serde_json::json!(b),
                Value::Timestamp(ts) => serde_json::json!(ts.to_rfc3339()),
                Value::Json(j) => serde_json::from_str(j).unwrap_or(serde_json::Value::Null),
            };
            args_array.push(js_value);
        }
        let args_array_json = serde_json::to_string(&args_array)
            .map_err(|e| Error::internal(format!("Failed to serialize arguments array: {}", e)))?;
        let set_args_script = format!("globalThis.arguments = {};", args_array_json);
        runtime
            .execute_script("<set_args>", set_args_script)
            .map_err(|e| Error::internal(format!("Failed to set arguments array: {}", e)))?;

        // Bind arguments as global variables using parameter names
        for (i, arg) in args.iter().enumerate() {
            let param_name = param_names[i];
            let js_value = match arg {
                Value::Null(_) => serde_json::Value::Null,
                Value::Integer(i) => serde_json::json!(i),
                Value::Float(f) => serde_json::json!(f),
                Value::Text(s) => serde_json::json!(s.as_ref()),
                Value::Boolean(b) => serde_json::json!(b),
                Value::Timestamp(ts) => serde_json::json!(ts.to_rfc3339()),
                Value::Json(j) => serde_json::from_str(j).unwrap_or(serde_json::Value::Null),
            };
            let set_script = format!("globalThis.{} = {};", param_name, js_value);
            runtime
                .execute_script("<set_arg>", set_script)
                .map_err(|e| {
                    Error::internal(format!("Failed to set argument {}: {}", param_name, e))
                })?;
        }

        // Execute the function call with named variables available
        let script = format!(
            "
            globalThis.__user_function = function() {{
                {}
            }};
            globalThis.__user_function.apply(null, globalThis.arguments)
        ",
            code
        );

        let result = runtime
            .execute_script("<user_function>", script)
            .map_err(|e| Error::internal(format!("Function execution failed: {}", e)))?;

        // Extract the result
        let scope = &mut runtime.handle_scope();
        let local = v8::Local::new(scope, result);

        // Try to deserialize as JSON first
        match serde_v8::from_v8::<serde_json::Value>(scope, local) {
            Ok(json_value) => match json_value {
                serde_json::Value::String(s) => Ok(Value::Text(s.into())),
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
                serde_json::Value::Null => Ok(Value::Null(crate::core::types::DataType::Null)),
                _ => Ok(Value::Json(json_value.to_string().into())),
            },
            Err(_) => {
                // Fallback: try to convert as string
                if local.is_string() {
                    let string = serde_v8::from_v8::<String>(scope, local).map_err(|e| {
                        Error::internal(format!("Failed to deserialize string result: {}", e))
                    })?;
                    Ok(Value::Text(string.into()))
                } else {
                    Err(Error::internal("Failed to deserialize function result"))
                }
            }
        }
    }

    fn validate_code(&self, _code: &str) -> Result<()> {
        // For now, we'll do basic validation by attempting to create a runtime
        // In the future, we could add proper AST parsing/validation
        let _runtime = JsRuntime::new(create_secure_runtime_options());
        // TODO: Add actual syntax validation
        Ok(())
    }
}

/// Stub implementation when Deno feature is not enabled
#[cfg(not(feature = "deno"))]
pub struct DenoBackend;

#[cfg(not(feature = "deno"))]
impl DenoBackend {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "deno"))]
impl ScriptingBackend for DenoBackend {
    fn name(&self) -> &'static str {
        "deno"
    }

    fn supported_languages(&self) -> &[&'static str] {
        &["deno", "javascript", "js", "typescript", "ts"]
    }

    fn execute(&self, _code: &str, _args: &[Value], _param_names: &[&str]) -> Result<Value> {
        Err(Error::internal(
            "Deno backend not enabled. Use --features deno to enable JavaScript/TypeScript support",
        ))
    }

    fn validate_code(&self, _code: &str) -> Result<()> {
        Err(Error::internal(
            "Deno backend not enabled. Use --features deno to enable JavaScript/TypeScript support",
        ))
    }
}
