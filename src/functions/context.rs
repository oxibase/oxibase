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

//! Execution Contexts
//!
//! This module provides thread-local contexts for functions to access out-of-band data,
//! such as HTTP request headers when a procedure is invoked via HTTP.

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// Thread-local storage for HTTP headers associated with the current execution.
    /// This is populated when a procedure is invoked via the HTTP API, allowing
    /// the `get_http_header` function to access request metadata.
    pub static HTTP_HEADERS: RefCell<Option<HashMap<String, String>>> = const { RefCell::new(None) };

    /// Thread-local storage for standard output captured during script execution.
    pub static STDOUT_CAPTURE: RefCell<String> = const { RefCell::new(String::new()) };

    /// Thread-local storage for the DebugController, allowing tracing hooks to pause execution.
    pub static DEBUG_CONTROLLER: RefCell<Option<std::sync::Arc<crate::common::debug::DebugController>>> = const { RefCell::new(None) };

    /// Thread-local storage for the currently executing procedure name.
    pub static CURRENT_PROCEDURE_NAME: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Executes a closure with HTTP headers and debug controller available in the thread-local context.
pub fn with_http_headers_and_debug<F, R>(
    headers: HashMap<String, String>,
    debug_controller: Option<std::sync::Arc<crate::common::debug::DebugController>>,
    f: F,
) -> R
where
    F: FnOnce() -> R,
{
    // Set the headers and debug controller
    HTTP_HEADERS.with(|h| *h.borrow_mut() = Some(headers));
    STDOUT_CAPTURE.with(|s| s.borrow_mut().clear());
    DEBUG_CONTROLLER.with(|c| *c.borrow_mut() = debug_controller);

    // Execute the closure
    let result = f();

    // Cleanup to avoid leaking context across different executions on the same thread
    HTTP_HEADERS.with(|h| *h.borrow_mut() = None);
    DEBUG_CONTROLLER.with(|c| *c.borrow_mut() = None);

    result
}

/// Sets the current procedure name
pub fn set_current_procedure_name(name: Option<String>) {
    CURRENT_PROCEDURE_NAME.with(|n| *n.borrow_mut() = name);
}

/// Gets the current procedure name
pub fn get_current_procedure_name() -> Option<String> {
    CURRENT_PROCEDURE_NAME.with(|n| n.borrow().clone())
}

/// Helper just for HTTP headers (for backward compatibility if needed)
pub fn with_http_headers<F, R>(headers: HashMap<String, String>, f: F) -> R
where
    F: FnOnce() -> R,
{
    with_http_headers_and_debug(headers, None, f)
}

/// Appends string to the stdout capture
pub fn append_stdout(s: &str) {
    STDOUT_CAPTURE.with(|out| {
        out.borrow_mut().push_str(s);
    });
}

/// Gets the current stdout capture
pub fn get_stdout() -> String {
    STDOUT_CAPTURE.with(|out| out.borrow().clone())
}

/// Gets the current debug controller
pub fn get_debug_controller() -> Option<std::sync::Arc<crate::common::debug::DebugController>> {
    DEBUG_CONTROLLER.with(|c| c.borrow().clone())
}
