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

use super::env::Environment;
use super::interpreter::PlSqlInterpreter;
use super::parser::PlSqlParser;
use crate::core::{Result, Value};
use crate::functions::backends::ScriptingBackend;
use crate::functions::FunctionRegistry;
use std::sync::Arc;

pub struct PlSqlBackend {
    function_registry: Arc<FunctionRegistry>,
}

impl PlSqlBackend {
    pub fn new(function_registry: Arc<FunctionRegistry>) -> Self {
        Self { function_registry }
    }
}

struct PlSqlDebugHook;

impl crate::functions::plsql::interpreter::DebugAdapterHook for PlSqlDebugHook {
    fn on_statement_before_eval(&self, line_number: usize, env: &Environment) {
        if let Some(proc_name) = crate::functions::context::get_current_procedure_name() {
            if let Some(dc) = crate::functions::context::get_debug_controller() {
                println!(
                    "PlSqlDebugHook: proc_name: {}, line_number: {}, has_breakpoint? {}",
                    proc_name,
                    line_number,
                    dc.has_breakpoint(&proc_name, line_number)
                );
                if dc.has_breakpoint(&proc_name, line_number) {
                    let mut local_map = serde_json::Map::new();
                    for scope in env.to_dap_scopes() {
                        for var in scope.variables {
                            local_map.insert(var.name, serde_json::Value::String(var.value));
                        }
                    }

                    let _ = dc.pause_execution(
                        line_number,
                        serde_json::Value::Object(local_map),
                        serde_json::Value::Object(serde_json::Map::new()),
                    );
                }
            }
        }
    }
}

impl ScriptingBackend for PlSqlBackend {
    fn name(&self) -> &'static str {
        "plsql"
    }

    fn supported_languages(&self) -> &[&'static str] {
        &["sql", "plsql", "pl/sql"]
    }

    fn validate_code(&self, code: &str) -> Result<()> {
        let mut parser = PlSqlParser::new(code);
        match parser.parse() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn execute(&self, code: &str, args: &[Value], param_names: &[&str]) -> Result<Value> {
        let mut parser = PlSqlParser::new(code);
        let block = parser.parse()?;

        let mut env = Environment::new();

        // Bind arguments globally
        for (i, arg) in args.iter().enumerate() {
            env.define_global(param_names[i], arg.clone());
        }

        let interpreter = PlSqlInterpreter::new(self.function_registry.clone(), None)
            .with_debug_hook(Arc::new(PlSqlDebugHook));

        if let Some(val) = interpreter.execute(&block, &mut env)? {
            Ok(val)
        } else {
            Ok(Value::Null(crate::core::DataType::Null))
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
        let mut parser = PlSqlParser::new(code);
        let block = parser.parse()?;

        let mut env = Environment::new();

        // Bind arguments globally
        for (i, arg) in args.iter().enumerate() {
            env.define_global(param_names[i], arg.clone());
        }

        let interpreter = PlSqlInterpreter::new(self.function_registry.clone(), _runner)
            .with_debug_hook(Arc::new(PlSqlDebugHook));
        interpreter.execute(&block, &mut env)?;

        // Write back OUT/INOUT values
        for (i, arg) in args.iter_mut().enumerate() {
            if let Some(val) = env.get(param_names[i]) {
                *arg = val.clone();
            } else {
                println!(
                    "Warning: {} not found in environment to write back",
                    param_names[i]
                );
            }
        }

        Ok(())
    }
}
