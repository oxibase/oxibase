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

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct DebugController {
    // Map of procedure names to a set of active line numbers for breakpoints
    breakpoints: Mutex<HashMap<String, HashSet<usize>>>,

    // Channel sender to transmit events (like "stopped") back to the active WebSocket client
    pub client_tx: Mutex<Option<mpsc::UnboundedSender<serde_json::Value>>>,

    // A notifier to pause execution threads.
    pause_condvar: Arc<std::sync::Condvar>,
    pub pause_mutex: Arc<Mutex<PauseState>>,
}

#[derive(Default)]
pub struct PauseState {
    pub is_paused: bool,
    pub resume_action: Option<ResumeAction>,
    // Store variables/state here temporarily while paused so DAP can query it
    pub current_locals: Option<serde_json::Value>,
    pub current_globals: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResumeAction {
    Continue,
    StepOver,
    Disconnect,
}

impl Default for DebugController {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugController {
    pub fn new() -> Self {
        Self {
            breakpoints: Mutex::new(HashMap::new()),
            client_tx: Mutex::new(None),
            pause_condvar: Arc::new(std::sync::Condvar::new()),
            pause_mutex: Arc::new(Mutex::new(PauseState::default())),
        }
    }

    pub fn set_breakpoints(&self, procedure_name: &str, lines: Vec<usize>) {
        let mut bps = self.breakpoints.lock().unwrap();
        let set: HashSet<usize> = lines.into_iter().collect();
        bps.insert(procedure_name.to_uppercase(), set);
    }

    pub fn has_breakpoint(&self, procedure_name: &str, line: usize) -> bool {
        let bps = self.breakpoints.lock().unwrap();
        if let Some(set) = bps.get(&procedure_name.to_uppercase()) {
            set.contains(&line)
        } else {
            false
        }
    }

    pub fn send_event(&self, event: serde_json::Value) {
        if let Some(tx) = self.client_tx.lock().unwrap().as_ref() {
            if let Err(e) = tx.send(event) {
                println!("Failed to send DAP event to client channel: {}", e);
            } else {
                println!("Sent DAP event to client channel");
            }
        } else {
            println!("No active client_tx to send DAP event");
        }
    }

    pub fn pause_execution(
        &self,
        _line: usize,
        locals: serde_json::Value,
        globals: serde_json::Value,
    ) -> ResumeAction {
        // Send stopped event to client
        let event = serde_json::json!({
            "seq": 0,
            "type": "event",
            "event": "stopped",
            "body": {
                "reason": "breakpoint",
                "threadId": 1,
                "allThreadsStopped": true
            }
        });
        self.send_event(event);

        let mut state = self.pause_mutex.lock().unwrap();
        state.is_paused = true;
        state.resume_action = None;
        state.current_locals = Some(locals);
        state.current_globals = Some(globals);

        // Block the current thread until the client sends a resume action
        while state.resume_action.is_none() {
            state = self.pause_condvar.wait(state).unwrap();
        }

        let action = state
            .resume_action
            .clone()
            .unwrap_or(ResumeAction::Continue);
        state.is_paused = false;
        state.current_locals = None;
        state.current_globals = None;
        action
    }

    pub fn resume(&self, action: ResumeAction) {
        let mut state = self.pause_mutex.lock().unwrap();
        state.resume_action = Some(action);
        self.pause_condvar.notify_all();
    }
}
