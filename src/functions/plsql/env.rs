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

/// A stack frame in the PL/SQL execution environment
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Variables defined in this scope
    variables: HashMap<String, Value>,
    /// Name of the frame (e.g., block, loop, procedure name)
    #[allow(dead_code)]
    name: String,
}

impl StackFrame {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            variables: HashMap::new(),
            name: name.into(),
        }
    }
}

/// The execution environment (stack frame) for PL/SQL execution
#[derive(Debug)]
pub struct Environment {
    /// Call stack frames for variables and scoping
    frames: Vec<StackFrame>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            frames: vec![StackFrame::new("global")],
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DapVariable {
    pub name: String,
    pub value: String,
    pub type_hint: String,
}

#[derive(Debug, Clone)]
pub struct DapScope {
    pub name: String,
    pub variables: Vec<DapVariable>,
}

impl Environment {
    /// Push a new frame onto the stack
    pub fn push_frame(&mut self, name: impl Into<String>) {
        self.frames.push(StackFrame::new(name));
    }

    /// Pop the current frame off the stack
    pub fn pop_frame(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    /// Define a variable in the current frame
    pub fn define(&mut self, name: &str, value: Value) {
        if let Some(frame) = self.frames.last_mut() {
            frame.variables.insert(name.trim().to_lowercase(), value);
        }
    }

    /// Define a variable in the root/global frame
    pub fn define_global(&mut self, name: &str, value: Value) {
        if let Some(frame) = self.frames.first_mut() {
            frame.variables.insert(name.trim().to_lowercase(), value);
        }
    }

    /// Assign a value to an existing variable, searching from inner to outer frames
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        let key = name.trim().to_lowercase();

        for frame in self.frames.iter_mut().rev() {
            if let std::collections::hash_map::Entry::Occupied(mut e) =
                frame.variables.entry(key.clone())
            {
                e.insert(value);
                return Ok(());
            }
        }

        Err(format!("Undefined variable '{}'", name))
    }

    /// Get a variable's value, searching from inner to outer frames
    pub fn get(&self, name: &str) -> Option<&Value> {
        let key = name.trim().to_lowercase();
        for frame in self.frames.iter().rev() {
            if let Some(val) = frame.variables.get(&key) {
                return Some(val);
            }
        }
        None
    }

    /// Exposed for future DAP support to inspect the stack
    pub fn frames(&self) -> &[StackFrame] {
        &self.frames
    }

    /// Converts current stack frames and variables to DAP structures
    pub fn to_dap_scopes(&self) -> Vec<DapScope> {
        self.frames
            .iter()
            .map(|f| {
                let mut variables = f
                    .variables
                    .iter()
                    .map(|(k, v)| DapVariable {
                        name: k.clone(),
                        value: match v {
                            Value::Text(t) => t.to_string(),
                            Value::Integer(i) => i.to_string(),
                            Value::Float(f) => f.to_string(),
                            Value::Boolean(b) => b.to_string(),
                            Value::Null(_) => "NULL".to_string(),
                            _ => format!("{:?}", v),
                        },
                        type_hint: format!("{:?}", v)
                            .split('(')
                            .next()
                            .unwrap_or("")
                            .to_string(),
                    })
                    .collect::<Vec<_>>();
                variables.sort_by(|a, b| a.name.cmp(&b.name));
                DapScope {
                    name: f.name.clone(),
                    variables,
                }
            })
            .collect()
    }
}
