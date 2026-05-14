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

//! Sequence Definitions

use crate::core::{Error, Result};
use std::sync::atomic::{AtomicI64, Ordering};

/// Configuration options for a Sequence
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceOptions {
    pub increment_by: i64,
    pub start_with: i64,
    pub min_value: i64,
    pub max_value: i64,
    pub cycle: bool,
}

impl Default for SequenceOptions {
    fn default() -> Self {
        Self {
            increment_by: 1,
            start_with: 1,
            min_value: 1,
            max_value: i64::MAX,
            cycle: false,
        }
    }
}

/// In-memory state of an active sequence
#[derive(Debug)]
pub struct SequenceState {
    current_value: AtomicI64,
    is_called: std::sync::atomic::AtomicBool,
    pub options: SequenceOptions,
}

impl SequenceState {
    pub fn new(options: SequenceOptions) -> Self {
        Self {
            current_value: AtomicI64::new(options.start_with),
            is_called: std::sync::atomic::AtomicBool::new(false),
            options,
        }
    }

    /// Increments the sequence and returns the new value safely across threads.
    pub fn nextval(&self) -> Result<i64> {
        // If not called yet, the first nextval returns the start_with value
        if !self.is_called.swap(true, Ordering::SeqCst) {
            return Ok(self.current_value.load(Ordering::SeqCst));
        }

        let mut current = self.current_value.load(Ordering::SeqCst);
        loop {
            let next = current.saturating_add(self.options.increment_by);

            // Handle bounds checking
            if self.options.increment_by > 0 && next > self.options.max_value {
                if self.options.cycle {
                    if let Err(actual) = self.current_value.compare_exchange_weak(
                        current,
                        self.options.min_value,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        current = actual;
                        continue;
                    }
                    return Ok(self.options.min_value);
                } else {
                    return Err(Error::internal("sequence exceeded max value"));
                }
            } else if self.options.increment_by < 0 && next < self.options.min_value {
                if self.options.cycle {
                    if let Err(actual) = self.current_value.compare_exchange_weak(
                        current,
                        self.options.max_value,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        current = actual;
                        continue;
                    }
                    return Ok(self.options.max_value);
                } else {
                    return Err(Error::internal("sequence exceeded min value"));
                }
            }

            match self.current_value.compare_exchange_weak(
                current,
                next,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return Ok(next),
                Err(actual) => current = actual,
            }
        }
    }

    pub fn setval(&self, value: i64, is_called: bool) -> Result<i64> {
        if value < self.options.min_value || value > self.options.max_value {
            return Err(Error::invalid_argument(format!(
                "value {} is out of bounds [{}, {}]",
                value, self.options.min_value, self.options.max_value
            )));
        }
        self.current_value.store(value, Ordering::SeqCst);
        self.is_called.store(is_called, Ordering::SeqCst);
        Ok(value)
    }

    pub fn current_value(&self) -> i64 {
        self.current_value.load(Ordering::SeqCst)
    }
}
