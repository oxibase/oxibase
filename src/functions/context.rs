// Copyright 2026 Stoolap Contributors
// Copyright 2026 Oxibase Contributors
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
}

/// Executes a closure with HTTP headers available in the thread-local context.
pub fn with_http_headers<F, R>(headers: HashMap<String, String>, f: F) -> R
where
    F: FnOnce() -> R,
{
    // Set the headers
    HTTP_HEADERS.with(|h| *h.borrow_mut() = Some(headers));

    // Execute the closure
    let result = f();

    // Cleanup to avoid leaking context across different executions on the same thread
    HTTP_HEADERS.with(|h| *h.borrow_mut() = None);

    result
}
