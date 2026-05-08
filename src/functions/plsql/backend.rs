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

use crate::core::{Error, Result, Value};
use crate::functions::backends::ScriptingBackend;
use super::parser::PlSqlParser;
use super::env::Environment;
use super::interpreter::PlSqlInterpreter;
use std::sync::Arc;
use crate::functions::FunctionRegistry;

pub struct PlSqlBackend {
    function_registry: Arc<FunctionRegistry>,
}

impl PlSqlBackend {
    pub fn new(function_registry: Arc<FunctionRegistry>) -> Self {
        Self { function_registry }
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

    fn execute(&self, code: &str, _args: &[Value], _param_names: &[&str]) -> Result<Value> {
        // Scalar functions in PL/SQL not fully implemented yet
        Err(Error::internal("PL/SQL scalar functions not implemented"))
    }

    fn execute_procedure(
        &self,
        code: &str,
        args: &mut [Value],
        param_names: &[&str],
        _modes: &[&str],
    ) -> Result<()> {
        let mut parser = PlSqlParser::new(code);
        let block = parser.parse()?;

        let mut env = Environment::new();

        // Bind arguments
        for (i, arg) in args.iter().enumerate() {
            env.define(param_names[i], arg.clone());
        }

        let interpreter = PlSqlInterpreter::new(self.function_registry.clone());
        interpreter.execute(&block, &mut env)?;

        // Write back OUT/INOUT values
        for (i, arg) in args.iter_mut().enumerate() {
            if let Some(val) = env.get(param_names[i]) {
                println!("Writing back {} = {:?}", param_names[i], val);
                *arg = val.clone();
            } else {
                println!("Warning: {} not found in environment to write back", param_names[i]);
            }
        }

        Ok(())
    }
}
