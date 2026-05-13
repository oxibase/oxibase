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

use crate::core::{Row, Schema};
use std::cell::{Cell, RefCell};

thread_local! {
    /// Pointer to the NEW row (used in INSERT, UPDATE)
    pub static CURRENT_NEW_ROW: RefCell<Option<*mut Row>> = const { RefCell::new(None) };
    /// Pointer to the OLD row (used in UPDATE, DELETE)
    pub static CURRENT_OLD_ROW: RefCell<Option<*const Row>> = const { RefCell::new(None) };
    /// Pointer to the table schema for looking up columns
    pub static CURRENT_SCHEMA: RefCell<Option<*const Schema>> = const { RefCell::new(None) };
    /// Prevents infinite recursion
    pub static TRIGGER_DEPTH_COUNTER: Cell<usize> = const { Cell::new(0) };
}

pub const MAX_TRIGGER_DEPTH: usize = 32;

/// Safely wraps the execution of a trigger by setting thread locals and clearing them afterwards.
pub fn with_trigger_context<F, R>(
    new_row: Option<&mut Row>,
    old_row: Option<&Row>,
    schema: &Schema,
    f: F,
) -> Result<R, crate::core::Error>
where
    F: FnOnce() -> Result<R, crate::core::Error>,
{
    let depth = TRIGGER_DEPTH_COUNTER.with(|c| c.get());
    if depth >= MAX_TRIGGER_DEPTH {
        return Err(crate::core::Error::internal(
            "Maximum trigger recursion depth exceeded",
        ));
    }

    TRIGGER_DEPTH_COUNTER.with(|c| c.set(depth + 1));
    CURRENT_SCHEMA.with(|s| *s.borrow_mut() = Some(schema as *const Schema));

    if let Some(new_r) = new_row {
        CURRENT_NEW_ROW.with(|r| *r.borrow_mut() = Some(new_r as *mut Row));
    }
    if let Some(old_r) = old_row {
        CURRENT_OLD_ROW.with(|r| *r.borrow_mut() = Some(old_r as *const Row));
    }

    // Execute the trigger
    let result = f();

    // Clean up
    CURRENT_NEW_ROW.with(|r| *r.borrow_mut() = None);
    CURRENT_OLD_ROW.with(|r| *r.borrow_mut() = None);
    CURRENT_SCHEMA.with(|s| *s.borrow_mut() = None);
    TRIGGER_DEPTH_COUNTER.with(|c| c.set(depth));

    result
}
