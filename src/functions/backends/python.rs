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
    compiler::Mode, convert::ToPyObject, AsObject, Interpreter, PyObjectRef, PyPayload, PyRef,
    Settings, VirtualMachine,
};

#[cfg(feature = "python")]
#[rustpython_vm::pymodule(name = "oxibase")]
mod oxibase_py_module {
    use rustpython_vm::{
        builtins::{PyIntRef, PyStrRef},
        PyResult, VirtualMachine,
    };

    #[pyfunction]
    fn execute(sql: PyStrRef, vm: &VirtualMachine) -> PyResult<PyIntRef> {
        match crate::functions::backends::execute_sql_query(sql.as_ref()) {
            Ok(res) => Ok(vm.ctx.new_int(res.rows_affected())),
            Err(e) => Err(vm.new_runtime_error(e.to_string())),
        }
    }

    #[pyfunction]
    fn query(sql: PyStrRef, vm: &VirtualMachine) -> PyResult<rustpython_vm::PyObjectRef> {
        match crate::functions::backends::execute_sql_query(sql.as_ref()) {
            Ok(mut res) => {
                let mut py_rows = Vec::new();
                let cols = res.columns().to_vec();
                while res.next() {
                    let py_dict = vm.ctx.new_dict();
                    for (i, col) in cols.iter().enumerate() {
                        let val = res
                            .row()
                            .get(i)
                            .cloned()
                            .unwrap_or(crate::core::Value::Null(crate::core::DataType::Null));
                        let py_val = match val {
                            crate::core::Value::Integer(v) => vm.ctx.new_int(v).into(),
                            crate::core::Value::Float(v) => vm.ctx.new_float(v).into(),
                            crate::core::Value::Text(v) => vm.ctx.new_str(v.to_string()).into(),
                            crate::core::Value::Boolean(v) => vm.ctx.new_bool(v).into(),
                            crate::core::Value::Timestamp(v) => {
                                vm.ctx.new_str(v.to_rfc3339()).into()
                            }
                            crate::core::Value::Null(_) => vm.ctx.none(),
                            crate::core::Value::Json(v) => vm.ctx.new_str(v.to_string()).into(),
                        };
                        let _ = py_dict.set_item(col.as_str(), py_val, vm);
                    }
                    py_rows.push(py_dict.into());
                }
                Ok(vm.ctx.new_list(py_rows).into())
            }
            Err(e) => Err(vm.new_runtime_error(e.to_string())),
        }
    }

    #[pyfunction]
    fn commit(vm: &VirtualMachine) -> PyResult<()> {
        match crate::functions::backends::commit_transaction() {
            Ok(_) => Ok(()),
            Err(e) => Err(vm.new_runtime_error(e.to_string())),
        }
    }

    #[pyfunction]
    fn rollback(vm: &VirtualMachine) -> PyResult<()> {
        match crate::functions::backends::rollback_transaction() {
            Ok(_) => Ok(()),
            Err(e) => Err(vm.new_runtime_error(e.to_string())),
        }
    }

    #[pyfunction]
    fn begin(vm: &VirtualMachine) -> PyResult<()> {
        match crate::functions::backends::begin_transaction() {
            Ok(_) => Ok(()),
            Err(e) => Err(vm.new_runtime_error(e.to_string())),
        }
    }

    #[pyfunction]
    fn log(level: PyStrRef, message: PyStrRef) {
        crate::common::logging::log_message(level.as_ref(), message.as_ref());
    }

    #[pyfunction]
    fn random(vm: &VirtualMachine) -> rustpython_vm::PyObjectRef {
        use rand::RngExt;
        use rustpython_vm::convert::ToPyObject;
        let val = rand::rng().random::<f64>();
        val.to_pyobject(vm)
    }

    #[pyfunction]
    fn get_http_header(
        header_name: PyStrRef,
        vm: &VirtualMachine,
    ) -> PyResult<rustpython_vm::PyObjectRef> {
        let mut header_value = None;
        crate::functions::context::HTTP_HEADERS.with(|headers| {
            if let Some(map) = headers.borrow().as_ref() {
                let search_key = header_name.to_string().to_lowercase();
                for (k, v) in map {
                    if k.to_lowercase() == search_key {
                        header_value = Some(v.clone());
                        break;
                    }
                }
            }
        });

        match header_value {
            Some(v) => Ok(vm.ctx.new_str(v).into()),
            None => Ok(vm.ctx.none()),
        }
    }

    #[pyfunction]
    fn _append_stdout(s: PyStrRef) {
        crate::functions::context::append_stdout(s.as_ref());
    }

    #[pyfunction]
    fn _check_breakpoint(
        line: usize,
        locals: rustpython_vm::builtins::PyDictRef,
        globals: rustpython_vm::builtins::PyDictRef,
        vm: &VirtualMachine,
    ) {
        if let Some(proc_name) = crate::functions::context::get_current_procedure_name() {
            if let Some(dc) = crate::functions::context::get_debug_controller() {
                if crate::functions::context::get_last_paused_line() != Some(line) {
                    crate::functions::context::set_last_paused_line(None);
                }

                let has_bp = dc.has_breakpoint(&proc_name, line);
                let is_stepping = crate::functions::context::get_is_stepping();
                let already_paused =
                    crate::functions::context::get_last_paused_line() == Some(line);

                println!(
                    "Python trace: line {}, has_bp={}, is_stepping={}, already_paused={}",
                    line, has_bp, is_stepping, already_paused
                );

                if (has_bp || is_stepping) && !already_paused {
                    let mut local_map = serde_json::Map::new();
                    for (k, v) in locals.into_iter() {
                        if let Ok(key) = k.str(vm) {
                            if let Ok(val) = v.str(vm) {
                                local_map.insert(
                                    key.to_string(),
                                    serde_json::Value::String(val.to_string()),
                                );
                            }
                        }
                    }
                    let mut global_map = serde_json::Map::new();
                    for (k, v) in globals.into_iter() {
                        if let Ok(key) = k.str(vm) {
                            if let Ok(val) = v.str(vm) {
                                global_map.insert(
                                    key.to_string(),
                                    serde_json::Value::String(val.to_string()),
                                );
                            }
                        }
                    }

                    let action = dc.pause_execution(
                        line,
                        serde_json::Value::Object(local_map),
                        serde_json::Value::Object(global_map),
                    );

                    crate::functions::context::set_last_paused_line(Some(line));

                    match action {
                        crate::common::debug::ResumeAction::Continue => {
                            crate::functions::context::set_is_stepping(false);
                        }
                        crate::common::debug::ResumeAction::StepOver => {
                            crate::functions::context::set_is_stepping(true);
                        }
                        crate::common::debug::ResumeAction::Disconnect => {
                            crate::functions::context::set_is_stepping(false);
                        }
                    }
                }
            }
        }
    }
}

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
    fn build_new_row_dict(
        &self,
        vm: &VirtualMachine,
    ) -> Result<rustpython_vm::builtins::PyDictRef> {
        let dict = vm.ctx.new_dict();

        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow() {
                        let row = unsafe { &*row_ptr };
                        for col in &schema.columns {
                            if let Some(val) = row.get(col.id) {
                                if let Ok(py_val) = self.convert_oxibase_to_python(val, vm) {
                                    let _ = dict.set_item(col.name.as_str(), py_val, vm);
                                }
                            }
                        }
                    }
                });
            }
        });

        Ok(dict)
    }

    fn build_old_row_dict(
        &self,
        vm: &VirtualMachine,
    ) -> Result<rustpython_vm::builtins::PyDictRef> {
        let dict = vm.ctx.new_dict();

        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow() {
                        let row = unsafe { &*row_ptr };
                        for col in &schema.columns {
                            if let Some(val) = row.get(col.id) {
                                if let Ok(py_val) = self.convert_oxibase_to_python(val, vm) {
                                    let _ = dict.set_item(col.name.as_str(), py_val, vm);
                                }
                            }
                        }
                    }
                });
            }
        });

        Ok(dict)
    }

    fn extract_new_row_dict(
        &self,
        dict: rustpython_vm::builtins::PyDictRef,
        vm: &VirtualMachine,
    ) -> Result<()> {
        let mut internal_err = None;
        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow_mut() {
                        let row = unsafe { &mut *row_ptr };
                        for col in &schema.columns {
                            if let Ok(py_val) = dict.get_item(col.name.as_str(), vm) {
                                match self.convert_python_to_oxibase(&py_val, vm) {
                                    Ok(v) => {
                                        let _ =
                                            row.set(col.id, v.into_coerce_to_type(col.data_type));
                                    }
                                    Err(e) => internal_err = Some(e),
                                }
                            }
                        }
                    }
                });
            }
        });

        if let Some(e) = internal_err {
            return Err(e);
        }
        Ok(())
    }

    /// Convert Oxibase Value to Python object
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

    /// Convert Python object back to Oxibase Value
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
        let builder = Interpreter::builder(Settings::default());
        let def = oxibase_py_module::module_def(&builder.ctx);
        let interpreter = builder.add_native_module(def).build();

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

                let redirect_code = r#"
import sys
import oxibase
class CaptureStdout:
    def write(self, s):
        oxibase._append_stdout(s)
    def flush(self):
        pass
sys.stdout = CaptureStdout()

def trace_hook(frame, event, arg):
    if event == "line":
        oxibase._check_breakpoint(frame.f_lineno, frame.f_locals, frame.f_globals)
    return trace_hook

sys.settrace(trace_hook)
"#;
                let _ = vm.run_string(scope.clone(), redirect_code, "<redirect>".to_string());

                // Execute the wrapper
                match vm.run_string(scope.clone(), &wrapper_code, "<user_function>".to_string()) {
                    Ok(_) => {
                        // Check for 'result' (set by the wrapper)
                        match scope.globals.get_item("result", vm) {
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

    fn execute_procedure(
        &self,
        code: &str,
        args: &mut [Value],
        param_names: &[&str],
        _modes: &[&str],
        _runner: Option<&dyn crate::functions::backends::SqlRunner>,
    ) -> Result<()> {
        let builder = Interpreter::builder(Settings::default());
        let def = oxibase_py_module::module_def(&builder.ctx);
        let interpreter = builder.add_native_module(def).build();

        interpreter
            .enter(|vm| {
                let scope = vm.new_scope_with_builtins();

                let mut oxibase_mod_opt = None;
                if let Ok(m) = vm.import("oxibase", 0) {
                    oxibase_mod_opt = Some(m);
                }

                if let Some(oxibase_mod) = oxibase_mod_opt {
                    let ctx_ns = rustpython_vm::builtins::PyNamespace {}.into_ref(&vm.ctx);
                    crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                        if r.borrow().is_some() {
                            if let Ok(dict) = self.build_new_row_dict(vm) {
                                use rustpython_vm::AsObject;
                                let _ = ctx_ns.as_object().set_attr("new", vm.new_pyobj(dict), vm);
                            }
                        }
                    });
                    crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
                        if r.borrow().is_some() {
                            if let Ok(dict) = self.build_old_row_dict(vm) {
                                use rustpython_vm::AsObject;
                                let _ = ctx_ns.as_object().set_attr("old", vm.new_pyobj(dict), vm);
                            }
                        }
                    });
                    let _ = oxibase_mod.set_attr("ctx", vm.new_pyobj(ctx_ns), vm);
                }

                for (i, arg) in args.iter().enumerate() {
                    let param_name = param_names[i];
                    let py_value = self.convert_oxibase_to_python(arg, vm)?;
                    scope
                        .globals
                        .set_item(param_name, py_value, vm)
                        .map_err(|e| {
                            Error::internal(format!(
                                "Failed to set parameter {}: {:?}",
                                param_name, e
                            ))
                        })?;
                }

                let redirect_code = r#"
import sys
import oxibase
class CaptureStdout:
    def write(self, s):
        oxibase._append_stdout(s)
    def flush(self):
        pass
sys.stdout = CaptureStdout()

def trace_hook(frame, event, arg):
    if event == "line":
        oxibase._check_breakpoint(frame.f_lineno, frame.f_locals, frame.f_globals)
    return trace_hook

sys.settrace(trace_hook)
"#;
                let _ = vm.run_string(scope.clone(), redirect_code, "<redirect>".to_string());

                match vm.compile(code, Mode::Exec, "<procedure>".to_string()) {
                    Ok(code_obj) => {
                        match vm.run_code_obj(code_obj, scope.clone()) {
                            Ok(_) => {
                                // Extract updated variables
                                for (i, arg) in args.iter_mut().enumerate() {
                                    let param_name = param_names[i];
                                    if let Ok(Some(py_val)) =
                                        scope.globals.get_item_opt(param_name, vm)
                                    {
                                        if let Ok(new_val) =
                                            self.convert_python_to_oxibase(&py_val, vm)
                                        {
                                            *arg = new_val;
                                        }
                                    }
                                }

                                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                                    if r.borrow().is_some() {
                                        let mut new_row_extracted = false;
                                        if let Ok(oxibase_mod) = vm.import("oxibase", 0) {
                                            if let Ok(ctx_obj) = oxibase_mod.get_attr("ctx", vm) {
                                                if let Ok(new_obj) = ctx_obj.get_attr("new", vm) {
                                                    if let Ok(dict) = new_obj.downcast::<rustpython_vm::builtins::PyDict>() {
                                                        let _ = self.extract_new_row_dict(dict, vm);
                                                        new_row_extracted = true;
                                                    }
                                                }
                                            }
                                        }
                                        if !new_row_extracted {
                                            // Fallback for scope extraction if needed?
                                            // Actually, FR-005 mandates reading from the nested path.
                                        }
                                    }
                                });

                                Ok(())
                            }
                            Err(py_err) => {
                                let error_msg = self.format_python_error(py_err, vm);
                                Err(Error::internal(format!(
                                    "Python execution error: {}",
                                    error_msg
                                )))
                            }
                        }
                    }
                    Err(py_err) => {
                        let error_msg = py_err.to_string();
                        Err(Error::internal(format!(
                            "Python compilation error: {}",
                            error_msg
                        )))
                    }
                }
            })
            .map_err(|e| Error::internal(format!("Interpreter error: {:?}", e)))?;
        Ok(())
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
