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
    AsObject,
    convert::ToPyObject,
    PyObjectRef, VirtualMachine,
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
    #[allow(dead_code)]
    fn convert_python_to_oxibase(&self, py_obj: &PyObjectRef, vm: &VirtualMachine) -> Result<Value> {
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

    fn execute(&self, _code: &str, _args: &[Value]) -> Result<Value> {
        // TODO: Implement Python execution
        // For now, return a placeholder - Python backend needs more work
        Err(Error::internal(
            "Python backend is not yet fully implemented",
        ))
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
