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

//! Python scripting backend for user-defined functions

use super::ScriptingBackend;
use crate::core::{Error, Result, Value};

#[cfg(feature = "python")]
use rustpython_vm::{
    builtins::PyInt, pyobject::PyObjectRef, Interpreter, PyResult, VirtualMachine,
};

/// Python scripting backend
#[cfg(feature = "python")]
pub struct PythonBackend {
    // Interpreter will be created per execution for isolation
}

#[cfg(feature = "python")]
impl PythonBackend {
    /// Create a new Python backend
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "python")]
impl ScriptingBackend for PythonBackend {
    fn name(&self) -> &'static str {
        "python"
    }

    fn supported_languages(&self) -> &[&'static str] {
        &["python", "py"]
    }

    fn execute(&self, code: &str, args: &[Value]) -> Result<Value> {
        Interpreter::with_init(Default::default(), |vm| {
            // Convert args to Python objects
            let py_args: Vec<PyObjectRef> = args
                .iter()
                .map(|arg| match arg {
                    Value::Integer(i) => vm.ctx.new_int(*i).into(),
                    Value::Float(f) => vm.ctx.new_float(*f).into(),
                    Value::Text(s) => vm.ctx.new_str(s.as_ref()).into(),
                    Value::Boolean(b) => vm.ctx.new_bool(*b).into(),
                    _ => vm.ctx.none(), // Default to None for unsupported types
                })
                .collect();

            // Create a list for arguments
            let args_list = vm.ctx.new_list(py_args);

            // Execute the function with arguments
            let script = format!(
                "
def __user_function(*args):
    {}

result = __user_function({})
                ",
                code, "args[0]" // For now, just pass first arg - need to expand
            );

            match vm.run_code_string(vm.ctx.new_scope_with_builtins(), &script, "<user_function>".to_string()) {
                Ok(_) => {
                    // Get the result from the scope
                    if let Some(result) = vm.get_attribute(vm.current_scope(), "result").ok() {
                        // Convert back to Value
                        if let Ok(int_val) = result.downcast::<PyInt>() {
                            Ok(Value::Integer(int_val.as_bigint().to_i64().unwrap_or(0)))
                        } else if let Ok(float_val) = result.downcast::<rustpython_vm::builtins::PyFloat>() {
                            Ok(Value::Float(float_val.to_f64()))
                        } else if let Ok(str_val) = result.downcast::<rustpython_vm::builtins::PyStr>() {
                            Ok(Value::Text(str_val.as_str().to_string().into()))
                        } else if let Ok(bool_val) = result.downcast::<rustpython_vm::builtins::PyBool>() {
                            Ok(Value::Boolean(bool_val.to_bool()))
                        } else {
                            Err(Error::internal("Unsupported return type from Python script"))
                        }
                    } else {
                        Err(Error::internal("No result variable found in Python script"))
                    }
                }
                Err(e) => Err(Error::internal(format!("Python execution error: {:?}", e))),
            }
        }).map_err(|e| Error::internal(format!("Python interpreter error: {:?}", e)))?
    }

    fn validate_code(&self, _code: &str) -> Result<()> {
        // TODO: Add Python syntax validation
        Ok(())
    }
}

/// Stub implementation when Python feature is not enabled
#[cfg(not(feature = "python"))]
pub struct PythonBackend;

#[cfg(not(feature = "python"))]
impl PythonBackend {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "python"))]
impl ScriptingBackend for PythonBackend {
    fn name(&self) -> &'static str {
        "python"
    }

    fn supported_languages(&self) -> &[&'static str] {
        &["python", "py"]
    }

    fn execute(&self, _code: &str, _args: &[Value]) -> Result<Value> {
        Err(Error::internal("Python backend not enabled. Use --features python to enable Python support"))
    }

    fn validate_code(&self, _code: &str) -> Result<()> {
        Err(Error::internal("Python backend not enabled. Use --features python to enable Python support"))
    }
}