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

use crate::storage::triggers::StoredTrigger;
use rustc_hash::FxHashMap;
use std::sync::RwLock;

/// In-memory cache of triggers for zero-overhead execution
pub struct TriggerRegistry {
    /// Maps a table name to its associated triggers
    tables: RwLock<FxHashMap<String, Vec<StoredTrigger>>>,
}

impl Default for TriggerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TriggerRegistry {
    pub fn new() -> Self {
        Self {
            tables: RwLock::new(FxHashMap::default()),
        }
    }

    /// Load all triggers into the registry
    pub fn load_triggers(&self, triggers: Vec<StoredTrigger>) {
        let mut map = FxHashMap::default();
        for trigger in triggers {
            map.entry(trigger.table_name.clone().to_uppercase())
                .or_insert_with(Vec::new)
                .push(trigger);
        }
        *self.tables.write().unwrap() = map;
    }

    /// Add a single trigger to the registry
    pub fn add_trigger(&self, trigger: StoredTrigger) {
        let mut map = self.tables.write().unwrap();
        map.entry(trigger.table_name.clone().to_uppercase())
            .or_insert_with(Vec::new)
            .push(trigger);
    }

    /// Remove a trigger by name
    pub fn remove_trigger(&self, trigger_name: &str) {
        let mut map = self.tables.write().unwrap();
        let trigger_name_upper = trigger_name.to_uppercase();
        
        for triggers in map.values_mut() {
            triggers.retain(|t| !t.name.eq_ignore_ascii_case(&trigger_name_upper));
        }
    }
    
    /// Remove all triggers for a specific table
    pub fn remove_table_triggers(&self, table_name: &str) {
        let mut map = self.tables.write().unwrap();
        map.remove(&table_name.to_uppercase());
    }

    /// Get all triggers for a specific table
    pub fn get_triggers(&self, table_name: &str) -> Vec<StoredTrigger> {
        let map = self.tables.read().unwrap();
        map.get(&table_name.to_uppercase()).cloned().unwrap_or_default()
    }
    
    /// Get BEFORE INSERT triggers
    pub fn get_before_insert(&self, table_name: &str) -> Vec<StoredTrigger> {
        self.get_filtered(table_name, "BEFORE", "INSERT")
    }

    /// Get AFTER INSERT triggers
    pub fn get_after_insert(&self, table_name: &str) -> Vec<StoredTrigger> {
        self.get_filtered(table_name, "AFTER", "INSERT")
    }

    /// Get BEFORE UPDATE triggers
    pub fn get_before_update(&self, table_name: &str) -> Vec<StoredTrigger> {
        self.get_filtered(table_name, "BEFORE", "UPDATE")
    }

    /// Get AFTER UPDATE triggers
    pub fn get_after_update(&self, table_name: &str) -> Vec<StoredTrigger> {
        self.get_filtered(table_name, "AFTER", "UPDATE")
    }

    /// Get BEFORE DELETE triggers
    pub fn get_before_delete(&self, table_name: &str) -> Vec<StoredTrigger> {
        self.get_filtered(table_name, "BEFORE", "DELETE")
    }

    /// Get AFTER DELETE triggers
    pub fn get_after_delete(&self, table_name: &str) -> Vec<StoredTrigger> {
        self.get_filtered(table_name, "AFTER", "DELETE")
    }

    fn get_filtered(&self, table_name: &str, timing: &str, event: &str) -> Vec<StoredTrigger> {
        let map = self.tables.read().unwrap();
        if let Some(triggers) = map.get(&table_name.to_uppercase()) {
            triggers
                .iter()
                .filter(|t| t.timing.eq_ignore_ascii_case(timing) && t.event.eq_ignore_ascii_case(event))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}
