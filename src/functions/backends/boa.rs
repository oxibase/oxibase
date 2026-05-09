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

//! Boa scripting backend for user-defined functions

use super::ScriptingBackend;
use crate::core::{Error, Result, Value};

use boa_engine::object::builtins::JsArray;
#[cfg(feature = "js")]
use boa_engine::{Context, JsString, JsValue, Source};

/// Boa scripting backend
#[cfg(feature = "js")]
pub struct BoaBackend {
    // Context will be created per execution for isolation
}

#[cfg(feature = "js")]
impl BoaBackend {
    /// Create a new Boa backend
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for BoaBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Create secure context options with filtered extensions
#[cfg(feature = "js")]
fn create_secure_context() -> Context {
    // Boa doesn't have built-in security restrictions like Deno
    // For now, we'll use default context and implement basic restrictions
    Context::default()
}

#[cfg(feature = "js")]
impl ScriptingBackend for BoaBackend {
    fn name(&self) -> &'static str {
        "boa"
    }

    fn supported_languages(&self) -> &[&'static str] {
        &["boa", "deno", "javascript", "js", "typescript", "ts"]
    }

    fn execute(&self, code: &str, args: &[Value], param_names: &[&str]) -> Result<Value> {
        // Create a new JavaScript context for each function call
        let mut context = create_secure_context();

        // Create arguments array for compatibility
        let js_array = JsArray::new(&mut context);
        for (i, arg) in args.iter().enumerate() {
            let js_value = match arg {
                Value::Null(_) => JsValue::null(),
                Value::Integer(i) => JsValue::new(*i as i32), // Boa uses i32 for integers
                Value::Float(f) => JsValue::rational(*f),
                Value::Text(s) => JsValue::new(JsString::from(s.as_ref())),
                Value::Boolean(b) => JsValue::new(*b),
                Value::Timestamp(ts) => JsValue::new(JsString::from(ts.to_rfc3339())),
                Value::Json(j) => JsValue::new(JsString::from(j.to_string())),
            };
            js_array
                .set(i as u32, js_value, false, &mut context)
                .map_err(|e| Error::internal(format!("Failed to set array element: {:?}", e)))?;
        }

        // Set arguments array as global
        context
            .register_global_property::<JsString, JsValue>(
                JsString::from("arguments"),
                js_array.into(),
                Default::default(),
            )
            .map_err(|e| Error::internal(format!("Failed to set arguments array: {}", e)))?;

        // Bind arguments as global variables using parameter names
        for (i, arg) in args.iter().enumerate() {
            let param_name = param_names[i];
            let js_value = match arg {
                Value::Null(_) => JsValue::null(),
                Value::Integer(i) => JsValue::new(*i as i32),
                Value::Float(f) => JsValue::rational(*f),
                Value::Text(s) => JsValue::new(JsString::from(s.as_ref())),
                Value::Boolean(b) => JsValue::new(*b),
                Value::Timestamp(ts) => JsValue::new(JsString::from(ts.to_rfc3339())),
                Value::Json(j) => JsValue::new(JsString::from(j.to_string())),
            };
            context
                .register_global_property::<JsString, JsValue>(
                    JsString::from(param_name),
                    js_value,
                    Default::default(),
                )
                .map_err(|e| {
                    Error::internal(format!("Failed to set argument {}: {}", param_name, e))
                })?;
        }

        // Execute the function call - create a function and call it with the arguments
        let wrapped_code = format!(
            r#"
            (function() {{
                {}
            }}).apply(null, arguments)
            "#,
            code
        );
        let source = Source::from_bytes(&wrapped_code);
        let result = context
            .eval(source)
            .map_err(|e| Error::internal(format!("Function execution failed: {:?}", e)))?;

        // Convert result back to Value
        if result.is_null() || result.is_undefined() {
            Ok(Value::Null(crate::core::types::DataType::Null))
        } else if let Some(b) = result.as_boolean() {
            Ok(Value::Boolean(b))
        } else if let Some(s) = result.as_string() {
            Ok(Value::Text(s.to_std_string().unwrap_or_default().into()))
        } else if let Some(i) = result.as_number() {
            if i.fract() == 0.0 && i >= i32::MIN as f64 && i <= i32::MAX as f64 {
                Ok(Value::Integer(i as i64))
            } else {
                Ok(Value::Float(i))
            }
        } else if let Some(bi) = result.as_bigint() {
            Ok(Value::Integer(bi.to_string().parse().unwrap_or(0)))
        } else {
            // For objects and other types, try to convert to JSON
            let json_value = result
                .to_json(&mut context)
                .unwrap_or(Some(serde_json::Value::String(
                    result.display().to_string(),
                )))
                .unwrap_or(serde_json::Value::String(result.display().to_string()));
            Ok(Value::Json(json_value.to_string().into()))
        }
    }

    fn validate_code(&self, code: &str) -> Result<()> {
        let mut context = create_secure_context();
        let source = Source::from_bytes(code);
        match boa_engine::Script::parse(source, None, &mut context) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::internal(format!("Boa syntax error: {:?}", e))),
        }
    }

    fn execute_procedure(
        &self,
        code: &str,
        args: &mut [Value],
        param_names: &[&str],
        _modes: &[&str],
        _runner: Option<&dyn crate::functions::backends::SqlRunner>,
    ) -> Result<()> {
        let mut context = create_secure_context();

        let oxibase_obj = boa_engine::object::ObjectInitializer::new(&mut context)
            .function(
                boa_engine::NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                    let sql = args
                        .get(0)
                        .cloned()
                        .unwrap_or_default()
                        .to_string(_ctx)
                        .unwrap_or_default()
                        .to_std_string_escaped();
                    
                    match crate::functions::backends::execute_sql_query(&sql) {
                        Ok(res) => Ok(JsValue::from(res.rows_affected() as i64)),
                        Err(e) => Err(boa_engine::JsError::from_opaque(JsValue::from(JsString::from(e.to_string())))),
                    }
                }),
                JsString::from("execute"),
                1,
            )
            .build();
            
        context
            .register_global_property(JsString::from("oxibase"), oxibase_obj, Default::default())
            .map_err(|e| Error::internal(format!("Failed to register JS oxibase: {}", e)))?;

        for (i, arg) in args.iter().enumerate() {
            let var_name = param_names[i];
            let js_val = match arg {
                Value::Integer(i) => JsValue::from(*i),
                Value::Float(f) => JsValue::from(*f),
                Value::Text(s) => JsValue::from(JsString::from(s.as_ref())),
                Value::Boolean(b) => JsValue::from(*b),
                Value::Null(_) => JsValue::null(),
                _ => return Err(Error::internal("Unsupported argument type for JS")),
            };
            context
                .register_global_property(JsString::from(var_name), js_val, Default::default())
                .map_err(|e| Error::internal(format!("Failed to register JS variable: {}", e)))?;
        }

        let source = Source::from_bytes(code);
        match context.eval(source) {
            Ok(_) => {
                for (i, arg) in args.iter_mut().enumerate() {
                    let var_name = param_names[i];
                    if let Ok(js_val) = context.global_object().get(JsString::from(var_name), &mut context) {
                        if js_val.is_number() {
                            if let Some(num) = js_val.as_number() {
                                if num.fract() == 0.0 {
                                    *arg = Value::Integer(num as i64);
                                } else {
                                    *arg = Value::Float(num);
                                }
                            }
                        } else if js_val.is_string() {
                            if let Some(s) = js_val.as_string() {
                                *arg = Value::Text(s.to_std_string_escaped().into());
                            }
                        } else if js_val.is_boolean() {
                            *arg = Value::Boolean(js_val.as_boolean().unwrap());
                        } else if js_val.is_null() || js_val.is_undefined() {
                            *arg = crate::core::value::Value::null_unknown();
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(Error::internal(format!("JS execution error: {}", e))),
        }
    }
}

/// Stub implementation when Boa feature is not enabled
#[cfg(not(feature = "js"))]
pub struct BoaBackend;

#[cfg(not(feature = "js"))]
impl BoaBackend {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "js"))]
impl ScriptingBackend for BoaBackend {
    fn name(&self) -> &'static str {
        "boa"
    }

    fn supported_languages(&self) -> &[&'static str] {
        &["boa", "deno", "javascript", "js", "typescript", "ts"]
    }

    fn execute(&self, _code: &str, _args: &[Value], _param_names: &[&str]) -> Result<Value> {
        Err(Error::internal(
            "Boa backend not enabled. Use --features js to enable JavaScript/TypeScript support",
        ))
    }

    fn validate_code(&self, _code: &str) -> Result<()> {
        Err(Error::internal(
            "Boa backend not enabled. Use --features js to enable JavaScript/TypeScript support",
        ))
    }
}
