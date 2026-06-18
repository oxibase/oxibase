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
use crate::core::{Error, Result, Value};
use rhai::{Engine, Scope};
use std::sync::Arc;

#[derive(Clone)]
pub struct RhaiDateTime(pub chrono::DateTime<chrono::Utc>);

impl RhaiDateTime {
    pub fn elapsed(&mut self) -> f64 {
        (chrono::Utc::now() - self.0).num_milliseconds() as f64 / 1000.0
    }
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&mut self) -> String {
        self.0.to_rfc3339()
    }
}

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

        engine.register_type_with_name::<RhaiDateTime>("DateTime");
        engine.register_fn("timestamp", || RhaiDateTime(chrono::Utc::now()));
        engine.register_fn("sleep", |ms: i64| {
            std::thread::sleep(std::time::Duration::from_millis(ms as u64))
        });
        engine.register_fn("elapsed", |dt: &mut RhaiDateTime| dt.elapsed());
        engine.register_fn("to_string", |dt: &mut RhaiDateTime| dt.to_string());

        engine.register_type_with_name::<NewRowProxy>("NewRowProxy");
        engine.register_indexer_get(|proxy: &mut NewRowProxy, prop: &str| proxy.get(prop));
        engine.register_indexer_set(|proxy: &mut NewRowProxy, prop: &str, val: rhai::Dynamic| {
            proxy.set(prop, val)
        });

        engine.register_type_with_name::<OldRowProxy>("OldRowProxy");
        engine.register_indexer_get(|proxy: &mut OldRowProxy, prop: &str| proxy.get(prop));

        let mut oxibase_module = rhai::Module::new();
        oxibase_module.set_native_fn(
            "execute",
            |sql: rhai::ImmutableString| -> std::result::Result<i64, Box<rhai::EvalAltResult>> {
                match crate::functions::backends::execute_sql_query(&sql) {
                    Ok(res) => Ok(res.rows_affected()),
                    Err(e) => Err(e.to_string().into()),
                }
            },
        );
        oxibase_module.set_native_fn(
            "get_http_header",
            |header_name: String| -> std::result::Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
                let mut header_value = None;
                crate::functions::context::HTTP_HEADERS.with(|headers| {
                    if let Some(map) = headers.borrow().as_ref() {
                        let search_key = header_name.to_lowercase();
                        for (k, v) in map {
                            if k.to_lowercase() == search_key {
                                header_value = Some(v.clone());
                                break;
                            }
                        }
                    }
                });

                match header_value {
                    Some(v) => Ok(rhai::Dynamic::from(v)),
                    None => Ok(rhai::Dynamic::UNIT),
                }
            },
        );
        oxibase_module.set_native_fn(
            "commit",
            || -> std::result::Result<(), Box<rhai::EvalAltResult>> {
                match crate::functions::backends::commit_transaction() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string().into()),
                }
            },
        );
        oxibase_module.set_native_fn(
            "rollback",
            || -> std::result::Result<(), Box<rhai::EvalAltResult>> {
                match crate::functions::backends::rollback_transaction() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string().into()),
                }
            },
        );
        oxibase_module.set_native_fn(
            "begin",
            || -> std::result::Result<(), Box<rhai::EvalAltResult>> {
                match crate::functions::backends::begin_transaction() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string().into()),
                }
            },
        );
        engine.register_static_module("oxibase", rhai::Shared::new(oxibase_module));

        engine.on_print(|x| {
            crate::functions::context::append_stdout(x);
        });

        #[cfg(debug_assertions)]
        #[allow(deprecated)]
        // since there is no standard debugging feature in our workspace, let's use debug build, or just remove the cfg
        engine.register_debugger(
            |_engine, debugger| debugger,
            |context: rhai::EvalContext,
             _event: rhai::debugger::DebuggerEvent,
             _node: rhai::ASTNode,
             _source: Option<&str>,
             pos: rhai::Position| {
                if let Some(line) = pos.line() {
                    if let Some(proc_name) = crate::functions::context::get_current_procedure_name()
                    {
                        if let Some(dc) = crate::functions::context::get_debug_controller() {
                            if dc.has_breakpoint(&proc_name, line) {
                                let mut local_map = serde_json::Map::new();
                                for (k, _, v) in context.scope().iter() {
                                    local_map.insert(
                                        k.to_string(),
                                        serde_json::Value::String(v.to_string()),
                                    );
                                }

                                let _ = dc.pause_execution(
                                    line,
                                    serde_json::Value::Object(local_map),
                                    serde_json::Value::Object(serde_json::Map::new()),
                                );
                            }
                        }
                    }
                }
                Ok(rhai::debugger::DebuggerCommand::Continue)
            },
        );

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

        let mut ctx_map = rhai::Map::new();
        crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
            if r.borrow().is_some() {
                ctx_map.insert("new".into(), rhai::Dynamic::from(NewRowProxy));
            }
        });
        crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
            if r.borrow().is_some() {
                ctx_map.insert("old".into(), rhai::Dynamic::from(OldRowProxy));
            }
        });
        let mut oxibase_map = rhai::Map::new();
        oxibase_map.insert("ctx".into(), rhai::Dynamic::from(ctx_map));
        scope.push("oxibase", oxibase_map);

        // Create arguments array for compatibility
        let mut args_array = rhai::Array::new();
        for arg in args {
            match arg {
                Value::Integer(i) => args_array.push(rhai::Dynamic::from(*i)),
                Value::Float(f) => args_array.push(rhai::Dynamic::from(*f)),
                Value::Text(s) => args_array.push(rhai::Dynamic::from(s.as_ref().to_string())),
                Value::Boolean(b) => args_array.push(rhai::Dynamic::from(*b)),
                Value::Timestamp(t) => args_array.push(rhai::Dynamic::from(RhaiDateTime(*t))),
                Value::Null(_) => args_array.push(rhai::Dynamic::UNIT),
                Value::Json(s) => {
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(s.as_ref()) {
                        args_array
                            .push(rhai::serde::to_dynamic(json_val).unwrap_or(rhai::Dynamic::UNIT));
                    } else {
                        args_array.push(rhai::Dynamic::from(s.as_ref().to_string()));
                    }
                }
            };
        }
        scope.push("arguments", args_array);

        // Bind arguments to scope using parameter names
        for (i, arg) in args.iter().enumerate() {
            let var_name = param_names[i];
            match arg {
                Value::Integer(i) => {
                    scope.push(var_name, *i);
                }
                Value::Float(f) => {
                    scope.push(var_name, *f);
                }
                Value::Text(s) => {
                    scope.push(var_name, s.as_ref().to_string());
                }
                Value::Boolean(b) => {
                    scope.push(var_name, *b);
                }
                Value::Timestamp(t) => {
                    scope.push(var_name, RhaiDateTime(*t));
                }
                Value::Null(_) => {
                    scope.push(var_name, ());
                }
                Value::Json(s) => {
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(s.as_ref()) {
                        let _ = scope.push_dynamic(
                            var_name,
                            rhai::serde::to_dynamic(json_val).unwrap_or(rhai::Dynamic::UNIT),
                        );
                    } else {
                        let _ = scope
                            .push_dynamic(var_name, rhai::Dynamic::from(s.as_ref().to_string()));
                    }
                }
            };
        }

        // Execute the script
        let processed = code
            .replace("oxibase.ctx.new", "oxibase.ctx[\"new\"]")
            .replace("oxibase.ctx.old", "oxibase.ctx[\"old\"]");
        match self
            .engine
            .eval_with_scope::<rhai::Dynamic>(&mut scope, &processed)
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
                } else if result.is::<RhaiDateTime>() {
                    Ok(Value::Timestamp(result.cast::<RhaiDateTime>().0))
                } else if result.is::<()>() {
                    Ok(Value::null_unknown())
                } else if result.is_map() || result.is_array() {
                    if let Ok(json_val) = rhai::serde::from_dynamic::<serde_json::Value>(&result) {
                        Ok(Value::Json(Arc::from(json_val.to_string())))
                    } else {
                        Ok(Value::Json(Arc::from(result.to_string())))
                    }
                } else {
                    Err(Error::internal("Unsupported return type from Rhai script"))
                }
            }
            Err(e) => Err(Error::internal(format!("Rhai execution error: {}", e))),
        }
    }

    fn validate_code(&self, code: &str) -> Result<()> {
        let processed = code
            .replace("oxibase.ctx.new", "oxibase.ctx[\"new\"]")
            .replace("oxibase.ctx.old", "oxibase.ctx[\"old\"]");
        match self.engine.compile(&processed) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::internal(format!("Rhai syntax error: {}", e))),
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
        let mut scope = Scope::new();

        let mut ctx_map = rhai::Map::new();
        crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
            if r.borrow().is_some() {
                ctx_map.insert("new".into(), rhai::Dynamic::from(NewRowProxy));
            }
        });
        crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
            if r.borrow().is_some() {
                ctx_map.insert("old".into(), rhai::Dynamic::from(OldRowProxy));
            }
        });
        let mut oxibase_map = rhai::Map::new();
        oxibase_map.insert("ctx".into(), rhai::Dynamic::from(ctx_map));
        scope.push("oxibase", oxibase_map);

        // Bind arguments to scope using parameter names
        for (i, arg) in args.iter().enumerate() {
            let var_name = param_names[i];
            match arg {
                Value::Integer(i) => {
                    scope.push(var_name, *i);
                }
                Value::Float(f) => {
                    scope.push(var_name, *f);
                }
                Value::Text(s) => {
                    scope.push(var_name, s.as_ref().to_string());
                }
                Value::Boolean(b) => {
                    scope.push(var_name, *b);
                }
                Value::Timestamp(t) => {
                    scope.push(var_name, RhaiDateTime(*t));
                }
                Value::Null(_) => {
                    scope.push(var_name, ());
                }
                Value::Json(s) => {
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(s.as_ref()) {
                        let _ = scope.push_dynamic(
                            var_name,
                            rhai::serde::to_dynamic(json_val).unwrap_or(rhai::Dynamic::UNIT),
                        );
                    } else {
                        let _ = scope
                            .push_dynamic(var_name, rhai::Dynamic::from(s.as_ref().to_string()));
                    }
                }
            };
        }

        // Execute the script
        let processed = code
            .replace("oxibase.ctx.new", "oxibase.ctx[\"new\"]")
            .replace("oxibase.ctx.old", "oxibase.ctx[\"old\"]");
        match self
            .engine
            .eval_with_scope::<rhai::Dynamic>(&mut scope, &processed)
        {
            Ok(_) => {
                // Read modified values back from scope
                for (i, arg) in args.iter_mut().enumerate() {
                    let var_name = param_names[i];
                    if let Some(val) = scope.get_value::<rhai::Dynamic>(var_name) {
                        if val.is::<i64>() {
                            *arg = Value::Integer(val.cast::<i64>());
                        } else if val.is::<f64>() {
                            *arg = Value::Float(val.cast::<f64>());
                        } else if val.is::<String>() {
                            *arg = Value::Text(val.cast::<String>().into());
                        } else if val.is::<bool>() {
                            *arg = Value::Boolean(val.cast::<bool>());
                        } else if val.is::<RhaiDateTime>() {
                            *arg = Value::Timestamp(val.cast::<RhaiDateTime>().0);
                        } else if val.is::<()>() {
                            *arg = Value::null_unknown();
                        } else if val.is_map() || val.is_array() {
                            if let Ok(json_val) =
                                rhai::serde::from_dynamic::<serde_json::Value>(&val)
                            {
                                *arg = Value::Json(Arc::from(json_val.to_string()));
                            } else {
                                *arg = Value::Json(Arc::from(val.to_string()));
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(Error::internal(format!("Rhai execution error: {}", e))),
        }
    }
}
// --- TRIGGER CONTEXT ---

#[derive(Clone)]
pub struct NewRowProxy;

#[derive(Clone)]
pub struct OldRowProxy;

impl NewRowProxy {
    pub fn get(
        &mut self,
        prop: &str,
    ) -> std::result::Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        let mut val = None;
        let mut found = false;

        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                if let Some(idx) = schema.get_column_index(prop) {
                    found = true;
                    crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                        if let Some(row_ptr) = *r.borrow() {
                            let row = unsafe { &*row_ptr };
                            if let Some(v) = row.get(idx) {
                                val = Some(crate::functions::backends::rhai::value_to_dynamic(v));
                            }
                        }
                    });
                }
            }
        });

        if !found {
            return Err(format!("Column not found: {}", prop).into());
        }

        Ok(val.unwrap_or(rhai::Dynamic::UNIT))
    }

    pub fn set(
        &mut self,
        prop: &str,
        new_val: rhai::Dynamic,
    ) -> std::result::Result<(), Box<rhai::EvalAltResult>> {
        let mut found = false;
        let mut success = false;
        let mut error = None;

        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                if let Some(idx) = schema.get_column_index(prop) {
                    found = true;
                    crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                        if let Some(row_ptr) = *r.borrow_mut() {
                            let row = unsafe { &mut *row_ptr };
                            if let Some(col) = schema.get_column(idx) {
                                match crate::functions::backends::rhai::dynamic_to_value(
                                    new_val.clone(),
                                    col.data_type,
                                ) {
                                    Ok(v) => {
                                        let _ = row.set(idx, v);
                                        success = true;
                                    }
                                    Err(e) => error = Some(e.to_string()),
                                }
                            }
                        }
                    });
                }
            }
        });

        if !found {
            return Err(format!("Column not found: {}", prop).into());
        }
        if let Some(err) = error {
            return Err(err.into());
        }

        Ok(())
    }
}

impl OldRowProxy {
    pub fn get(
        &mut self,
        prop: &str,
    ) -> std::result::Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        let mut val = None;
        let mut found = false;

        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                if let Some(idx) = schema.get_column_index(prop) {
                    found = true;
                    crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
                        if let Some(row_ptr) = *r.borrow() {
                            let row = unsafe { &*row_ptr };
                            if let Some(v) = row.get(idx) {
                                val = Some(crate::functions::backends::rhai::value_to_dynamic(v));
                            }
                        }
                    });
                }
            }
        });

        if !found {
            return Err(format!("Column not found: {}", prop).into());
        }

        Ok(val.unwrap_or(rhai::Dynamic::UNIT))
    }
}

pub(crate) fn value_to_dynamic(val: &crate::core::Value) -> rhai::Dynamic {
    match val {
        crate::core::Value::Timestamp(t) => rhai::Dynamic::from(RhaiDateTime(*t)),
        crate::core::Value::Integer(i) => rhai::Dynamic::from(*i),
        crate::core::Value::Float(f) => rhai::Dynamic::from(*f),
        crate::core::Value::Text(s) => rhai::Dynamic::from(s.as_ref().to_string()),
        crate::core::Value::Boolean(b) => rhai::Dynamic::from(*b),
        crate::core::Value::Null(_) => rhai::Dynamic::UNIT,
        crate::core::Value::Json(s) => {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(s.as_ref()) {
                rhai::serde::to_dynamic(json_val).unwrap_or(rhai::Dynamic::UNIT)
            } else {
                rhai::Dynamic::from(s.as_ref().to_string())
            }
        }
    }
}

pub(crate) fn dynamic_to_value(
    val: rhai::Dynamic,
    dt: crate::core::DataType,
) -> std::result::Result<crate::core::Value, crate::core::Error> {
    if val.is_unit() {
        return Ok(crate::core::Value::Null(dt));
    }

    match dt {
        crate::core::DataType::Integer => {
            if val.is::<i64>() {
                Ok(crate::core::Value::Integer(val.cast::<i64>()))
            } else if val.is::<i32>() {
                Ok(crate::core::Value::Integer(val.cast::<i32>() as i64))
            } else {
                Ok(crate::core::Value::Integer(val.as_int().map_err(|_| {
                    crate::core::Error::internal("Cannot cast to integer")
                })?))
            }
        }
        crate::core::DataType::Float => {
            if val.is::<f64>() {
                Ok(crate::core::Value::Float(val.cast::<f64>()))
            } else if val.is::<f32>() {
                Ok(crate::core::Value::Float(val.cast::<f32>() as f64))
            } else {
                Ok(crate::core::Value::Float(val.as_float().map_err(|_| {
                    crate::core::Error::internal("Cannot cast to float")
                })?))
            }
        }
        crate::core::DataType::Text => Ok(crate::core::Value::text(val.to_string())),
        crate::core::DataType::Boolean => {
            if val.is::<bool>() {
                Ok(crate::core::Value::Boolean(val.cast::<bool>()))
            } else {
                Ok(crate::core::Value::Boolean(val.as_bool().map_err(
                    |_| crate::core::Error::internal("Cannot cast to bool"),
                )?))
            }
        }
        crate::core::DataType::Timestamp => {
            if val.is::<RhaiDateTime>() {
                Ok(crate::core::Value::Timestamp(val.cast::<RhaiDateTime>().0))
            } else if val.is::<String>() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&val.cast::<String>()) {
                    Ok(crate::core::Value::Timestamp(
                        dt.with_timezone(&chrono::Utc),
                    ))
                } else {
                    Ok(crate::core::Value::Timestamp(chrono::Utc::now())) // Ponytail: naive fallback
                }
            } else {
                Err(crate::core::Error::internal("Cannot cast to timestamp")) // This will fail compilation, need standard Result
            }
        }
        crate::core::DataType::Json => {
            if val.is_map() || val.is_array() {
                if let Ok(json_val) = rhai::serde::from_dynamic::<serde_json::Value>(&val) {
                    Ok(crate::core::Value::Json(Arc::from(json_val.to_string())))
                } else {
                    Ok(crate::core::Value::Json(Arc::from(val.to_string())))
                }
            } else {
                Ok(crate::core::Value::Json(Arc::from(val.to_string())))
            }
        }
        _ => Ok(crate::core::Value::text(val.to_string())),
    }
}
