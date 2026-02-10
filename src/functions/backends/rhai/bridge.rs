// Copyright 2025 Stoolap Contributors
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

//! Rhai database bridge implementation using thread-local context
//!
//! Provides the 'db' interface to Rhai scripts for transactional SQL execution.

use crate::core::Value;
use crate::storage::traits::Transaction;
use rhai::{Dynamic, Engine, ImmutableString};
use std::cell::Cell;

thread_local! {
    /// Active transaction data pointer
    static ACTIVE_TXN_DATA: Cell<usize> = const { Cell::new(0) };
    /// Active transaction vtable pointer
    static ACTIVE_TXN_VTABLE: Cell<usize> = const { Cell::new(0) };
}

/// Guard to ensure the active transaction is cleared after execution
pub struct TxnGuard;

impl TxnGuard {
    /// Create a new guard for the given transaction
    pub fn new(txn: &dyn Transaction) -> Self {
        // Deconstruct the fat pointer manually
        let ptr = txn as *const dyn Transaction;
        let (data, vtable): (usize, usize) = unsafe { std::mem::transmute(ptr) };

        ACTIVE_TXN_DATA.with(|slot| slot.set(data));
        ACTIVE_TXN_VTABLE.with(|slot| slot.set(vtable));
        Self
    }
}

impl Drop for TxnGuard {
    fn drop(&mut self) {
        ACTIVE_TXN_DATA.with(|slot| slot.set(0));
        ACTIVE_TXN_VTABLE.with(|slot| slot.set(0));
    }
}

/// Internal helper to get the active transaction
fn get_active_txn() -> Option<&'static dyn Transaction> {
    let data = ACTIVE_TXN_DATA.with(|slot| slot.get());
    let vtable = ACTIVE_TXN_VTABLE.with(|slot| slot.get());

    if data == 0 || vtable == 0 {
        None
    } else {
        // Reconstruct the fat pointer
        let ptr: *const dyn Transaction = unsafe { std::mem::transmute((data, vtable)) };
        // SAFETY: The TxnGuard ensures this pointer is valid for the duration of the call.
        Some(unsafe { &*ptr })
    }
}

/// Register database functions in the Rhai engine
pub fn register_db_functions(engine: &mut Engine) {
    // Register 'db_query'
    engine.register_fn("db_query", |_sql: ImmutableString| -> rhai::Array {
        if let Some(_txn) = get_active_txn() {
            // TODO: Execute SQL via txn
            rhai::Array::new()
        } else {
            rhai::Array::new()
        }
    });

    // Register 'db_exec'
    engine.register_fn("db_exec", |_sql: ImmutableString| -> i64 {
        if let Some(_txn) = get_active_txn() {
            // TODO: Execute SQL via txn
            0
        } else {
            0
        }
    });
}

/// Convert OxiBase Value to Rhai Dynamic
pub fn value_to_dynamic(value: &Value) -> Dynamic {
    match value {
        Value::Integer(i) => Dynamic::from(*i),
        Value::Float(f) => Dynamic::from(*f),
        Value::Text(s) => Dynamic::from(s.as_ref().to_string()),
        Value::Boolean(b) => Dynamic::from(*b),
        Value::Timestamp(ts) => Dynamic::from(ts.to_rfc3339()),
        Value::Json(j) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(j.as_ref()) {
                match v {
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Dynamic::from(i)
                        } else if let Some(f) = n.as_f64() {
                            Dynamic::from(f)
                        } else {
                            Dynamic::UNIT
                        }
                    }
                    serde_json::Value::String(s) => Dynamic::from(s),
                    serde_json::Value::Bool(b) => Dynamic::from(b),
                    _ => Dynamic::from(j.as_ref().to_string()),
                }
            } else {
                Dynamic::from(j.as_ref().to_string())
            }
        }
        Value::Null(_) => Dynamic::UNIT,
    }
}
