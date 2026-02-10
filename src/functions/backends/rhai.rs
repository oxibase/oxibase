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

mod bridge;

use super::ScriptingBackend;
use crate::core::{Error, Result, Value};
use crate::storage::traits::Transaction;
use rhai::{Engine, Scope};

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

        // Register database bridge functions
        bridge::register_db_functions(&mut engine);

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

    fn execute(
        &self,
        code: &str,
        args: &[Value],
        param_names: &[&str],
        txn: Option<&dyn Transaction>,
    ) -> Result<Value> {
        // Set up the transaction guard if a transaction is provided
        let _guard = txn.map(bridge::TxnGuard::new);

        let mut scope = Scope::new();

        // Create arguments array for compatibility
        let mut args_array = rhai::Array::new();
        for arg in args {
            args_array.push(bridge::value_to_dynamic(arg));
        }
        scope.push("arguments", args_array);

        // Bind arguments to scope using parameter names
        for (i, arg) in args.iter().enumerate() {
            let var_name = param_names[i];
            scope.push(var_name, bridge::value_to_dynamic(arg));
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
                    Ok(Value::Null(crate::core::types::DataType::Null))
                }
            }
            Err(e) => Err(Error::internal(format!("Rhai execution error: {}", e))),
        }
    }

    fn validate_code(&self, code: &str) -> Result<()> {
        match self.engine.compile(code) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::internal(format!("Rhai syntax error: {}", e))),
        }
    }
}
