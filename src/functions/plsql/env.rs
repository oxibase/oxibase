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

use crate::core::Value;
use std::collections::HashMap;

/// The execution environment (stack frame) for PL/SQL execution
#[derive(Debug)]
pub struct Environment {
    /// Local variables in the current scope
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn define(&mut self, name: &str, value: Value) {
        self.variables.insert(name.trim().to_lowercase(), value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        let key = name.trim().to_lowercase();
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.variables.entry(key) {
            e.insert(value);
            Ok(())
        } else {
            Err(format!("Undefined variable '{}'", name))
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(&name.trim().to_lowercase())
    }
}
