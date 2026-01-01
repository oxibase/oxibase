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
    compiler::Mode, convert::ToPyObject, AsObject, Interpreter, PyObjectRef, PyRef, Settings,
    VirtualMachine,
};

/// Python scripting backend
#[cfg(feature = "python")]
pub struct PythonBackend {
    // Interpreter will be created per execution for isolation
}

#[cfg(feature = "python")]
impl PythonBackend {
    /// Indent each line of the code by 4 spaces for function wrapping
    fn indent_code(&self, code: &str) -> String {
        code.lines()
            .map(|line| format!("    {}", line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(feature = "python")]
impl PythonBackend {
    /// Create a new Python backend
    pub fn new() -> Self {
        Self {}
    }

    /// Convert OxiBase Value to Python object
    #[allow(dead_code)]
    fn convert_oxibase_to_python(&self, value: &Value, vm: &VirtualMachine) -> Result<PyObjectRef> {
        match value {
            Value::Null(_) => Ok(vm.ctx.none()),
            Value::Integer(i) => Ok(i.to_pyobject(vm)),
            Value::Float(f) => Ok(f.to_pyobject(vm)),
            Value::Text(s) => Ok(s.as_ref().to_pyobject(vm)),
            Value::Boolean(b) => Ok(b.to_pyobject(vm)),
            Value::Timestamp(ts) => {
                // Convert to Python datetime - simplified approach
                // For now, convert to ISO string and let Python handle it
                let iso_str = ts.to_rfc3339();
                Ok(iso_str.to_pyobject(vm))
            }
            Value::Json(j) => {
                // For now, just pass as string
                Ok(j.as_ref().to_pyobject(vm))
            }
        }
    }

    /// Convert Python object back to OxiBase Value
    fn convert_python_to_oxibase(
        &self,
        py_obj: &PyObjectRef,
        vm: &VirtualMachine,
    ) -> Result<Value> {
        if py_obj.is(&vm.ctx.none()) {
            return Ok(Value::null_unknown());
        }

        // Try to extract as different types using str() and parsing
        if let Ok(str_repr) = py_obj.str(vm) {
            // Convert PyStr to String
            let s = str_repr.to_string();
            // Try to parse as different types
            if let Ok(i) = s.parse::<i64>() {
                return Ok(Value::Integer(i));
            }
            if let Ok(f) = s.parse::<f64>() {
                return Ok(Value::Float(f));
            }
            if s == "True" {
                return Ok(Value::Boolean(true));
            }
            if s == "False" {
                return Ok(Value::Boolean(false));
            }
            // For strings and complex objects, return as text or JSON
            return Ok(Value::Text(s.into()));
        }

        // Fallback
        Err(Error::internal("Failed to convert Python object"))
    }

    /// Format Python error with basic error message
    fn format_python_error(
        &self,
        py_err: PyRef<rustpython_vm::builtins::PyBaseException>,
        vm: &VirtualMachine,
    ) -> String {
        // Format error consistently with other backends - try to get string representation
        match py_err.as_object().str(vm) {
            Ok(s) => s.to_string(),
            Err(_) => format!("{:?}", py_err),
        }
    }
}

impl Default for PythonBackend {
    fn default() -> Self {
        Self::new()
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

    fn execute(&self, code: &str, args: &[Value], param_names: &[&str]) -> Result<Value> {
        // Create a new interpreter for each execution (isolation)
        let interpreter = Interpreter::with_init(Settings::default(), |_| ());

        interpreter
            .enter(|vm| {
                let scope = vm.new_scope_with_builtins();

                // Create arguments list for compatibility
                let mut args_vec = Vec::new();
                for arg in args {
                    let py_value = self.convert_oxibase_to_python(arg, vm)?;
                    args_vec.push(py_value);
                }
                let args_list = vm.ctx.new_list(args_vec);
                scope.globals.set_item("arguments", args_list.into(), vm).map_err(|e| {
                    Error::internal(format!("Failed to set arguments list: {:?}", e))
                })?;

                // Convert arguments to Python variables using parameter names
                for (i, arg) in args.iter().enumerate() {
                    let param_name = param_names[i];
                    let py_value = self.convert_oxibase_to_python(arg, vm)?;
                    scope
                        .globals
                        .set_item(param_name, py_value, vm)
                        .map_err(|e| Error::internal(format!("Failed to set argument {}: {:?}", param_name, e)))?;
                }

                // Wrap user code in a function to support 'return' statements
                let indented_code = self.indent_code(code);
                let wrapper_code = format!(
                    "def __user_function():\n{}\nresult = __user_function()",
                    indented_code
                );

                // Execute the wrapper
                match vm.run_code_string(scope.clone(), &wrapper_code, "<user_function>".to_string()) {
                    Ok(_) => {
                        // Check for 'result' (set by the wrapper)
                        match scope.locals.get_item("result", vm) {
                            Ok(result) => self.convert_python_to_oxibase(&result, vm),
                            Err(_) => Err(Error::internal(
                                "Python function did not return a value (use 'return' or set 'result')",
                            )),
                        }
                    }
                    Err(py_err) => {
                        // Convert Python exception to detailed error message
                        let error_msg = self.format_python_error(py_err, vm);
                        Err(Error::internal(format!(
                            "Python execution error: {}",
                            error_msg
                        )))
                    }
                }
            })
            .map_err(|e| Error::internal(format!("Interpreter error: {:?}", e)))
    }

    fn validate_code(&self, code: &str) -> Result<()> {
        // Basic syntax validation using rustpython parsing with wrapper
        let interpreter = Interpreter::with_init(Settings::default(), |_| ());
        interpreter
            .enter(|vm| {
                // Wrap code in function for validation
                let indented_code = self.indent_code(code);
                let wrapper_code = format!(
                    "def __user_function():\n{}\nresult = __user_function()",
                    indented_code
                );

                // Try to compile the wrapped code to check for syntax errors
                match vm.compile(&wrapper_code, Mode::Exec, "<validation>".to_string()) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Error::internal(format!("Python syntax error: {}", e))),
                }
            })
            .map_err(|e| Error::internal(format!("Validation error: {:?}", e)))
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

    fn execute(&self, _code: &str, _args: &[Value], _param_names: &[&str]) -> Result<Value> {
        Err(Error::internal(
            "Python backend not enabled. Use --features python to enable Python support",
        ))
    }

    fn validate_code(&self, _code: &str) -> Result<()> {
        Err(Error::internal(
            "Python backend not enabled. Use --features python to enable Python support",
        ))
    }
}
