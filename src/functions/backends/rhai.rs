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

//! Rhai scripting backend for user-defined functions

use super::ScriptingBackend;
use crate::api::DatabaseOps;
use crate::core::{Error, Result, Value};
use rhai::{Dynamic, Engine, Scope};

/// Rhai scripting backend
pub struct RhaiBackend {
    engine: Engine,
}

impl RhaiBackend {
    /// Create a new Rhai backend
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Register custom functions for type conversions
        engine.register_fn("to_int", |v: i64| v);
        engine.register_fn("to_float", |v: f64| v);
        engine.register_fn("to_string", |v: String| v);

        Self { engine }
    }
}

impl Default for RhaiBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptingBackend for RhaiBackend {
    fn name(&self) -> &'static str {
        "rhai"
    }

    fn supported_languages(&self) -> &[&'static str] {
        &["rhai"]
    }

    fn execute(&self, code: &str, args: &[Value], param_names: &[&str]) -> Result<Value> {
        let mut scope = Scope::new();

        // Create arguments array for compatibility
        let mut args_array = rhai::Array::new();
        for arg in args {
            match arg {
                Value::Integer(i) => args_array.push(rhai::Dynamic::from(*i)),
                Value::Float(f) => args_array.push(rhai::Dynamic::from(*f)),
                Value::Text(s) => args_array.push(rhai::Dynamic::from(s.as_ref().to_string())),
                Value::Boolean(b) => args_array.push(rhai::Dynamic::from(*b)),
                _ => return Err(Error::internal("Unsupported argument type for Rhai")),
            };
        }
        scope.push("arguments", args_array);

        // Bind arguments to scope using parameter names
        for (i, arg) in args.iter().enumerate() {
            let var_name = param_names[i];
            match arg {
                Value::Integer(i) => scope.push(var_name, *i),
                Value::Float(f) => scope.push(var_name, *f),
                Value::Text(s) => scope.push(var_name, s.as_ref().to_string()),
                Value::Boolean(b) => scope.push(var_name, *b),
                _ => return Err(Error::internal("Unsupported argument type for Rhai")),
            };
        }

        // Execute the script
        match self
            .engine
            .eval_with_scope::<rhai::Dynamic>(&mut scope, code)
        {
            Ok(result) => {
                // Convert Rhai result back to Value
                if result.is::<i64>() {
                    Ok(Value::Integer(result.cast::<i64>()))
                } else if result.is::<f64>() {
                    Ok(Value::Float(result.cast::<f64>()))
                } else if result.is::<String>() {
                    Ok(Value::Text(result.cast::<String>().into()))
                } else if result.is::<bool>() {
                    Ok(Value::Boolean(result.cast::<bool>()))
                } else {
                    Err(Error::internal("Unsupported return type from Rhai script"))
                }
            }
            Err(e) => Err(Error::internal(format!("Rhai execution error: {}", e))),
        }
    }

    fn execute_procedure(
        &self,
        code: &str,
        args: &[Value],
        param_names: &[&str],
        db: Box<dyn DatabaseOps>,
    ) -> Result<()> {
        let mut scope = Scope::new();

        // Create a custom engine with the execute function
        let mut engine = Engine::new();

        // Register custom functions for type conversions (same as main engine)
        engine.register_fn("to_int", |v: i64| v);
        engine.register_fn("to_float", |v: f64| v);
        engine.register_fn("to_string", |v: String| v);

        // Use the database for the closure
        let db = std::sync::Arc::new(std::sync::Mutex::new(db));
        engine.register_fn(
            "execute",
            move |sql: &str| -> std::result::Result<Dynamic, Box<rhai::EvalAltResult>> {
                // Determine query type and execute accordingly
                let sql_upper = sql.trim_start().to_uppercase();
                if sql_upper.starts_with("SELECT") {
                    // SELECT queries return rows
                    match db.lock().unwrap().query(sql, ()) {
                        Ok(rows_iter) => {
                            let mut result_array = rhai::Array::new();
                            for row_result in rows_iter {
                                let row = match row_result {
                                    Ok(r) => r,
                                    Err(e) => return Err(format!("SQL row error: {}", e).into()),
                                };
                                let mut row_map = rhai::Map::new();
                                for (i, column) in row.columns().iter().enumerate() {
                                    let value = match row.get::<Value>(i) {
                                        Ok(val) => match val {
                                            Value::Integer(i) => Dynamic::from(i),
                                            Value::Float(f) => Dynamic::from(f),
                                            Value::Text(s) => Dynamic::from(s.as_ref().to_string()),
                                            Value::Boolean(b) => Dynamic::from(b),
                                            _ => Dynamic::from(()), // null/unknown
                                        },
                                        Err(_) => Dynamic::from(()),
                                    };
                                    row_map.insert(column.clone().into(), value);
                                }
                                result_array.push(Dynamic::from(row_map));
                            }
                            Ok(Dynamic::from(result_array))
                        }
                        Err(e) => Err(format!("SQL execution error: {}", e).into()),
                    }
                } else {
                    // DML/DDL queries return affected row count or success
                    match db.lock().unwrap().execute(sql, ()) {
                        Ok(_) => Ok(Dynamic::from(())),
                        Err(e) => Err(format!("SQL execution error: {}", e).into()),
                    }
                }
            },
        );

        // Bind arguments to scope using parameter names
        for (i, arg) in args.iter().enumerate() {
            let var_name = param_names[i];
            match arg {
                Value::Integer(i) => scope.push(var_name, *i),
                Value::Float(f) => scope.push(var_name, *f),
                Value::Text(s) => scope.push(var_name, s.as_ref().to_string()),
                Value::Boolean(b) => scope.push(var_name, *b),
                _ => {
                    return Err(Error::internal(
                        "Unsupported argument type for Rhai procedure",
                    ))
                }
            };
        }

        // Execute the procedure code
        match engine.eval_with_scope::<rhai::Dynamic>(&mut scope, code) {
            Ok(result) => {
                if result.is_unit() {
                    Ok(())
                } else {
                    Err(Error::internal("Procedure did not return unit"))
                }
            }
            Err(e) => Err(Error::internal(format!(
                "Rhai procedure execution error: {}",
                e
            ))),
        }
    }

    fn validate_code(&self, code: &str) -> Result<()> {
        match self.engine.compile(code) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::internal(format!("Rhai syntax error: {}", e))),
        }
    }
}
