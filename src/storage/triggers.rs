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

//! Triggers persistence
//!
//! This module provides storage for event triggers in system tables.
//! Triggers are stored in the `_sys_triggers` system table and loaded
//! into memory during database startup.

use serde::{Deserialize, Serialize};

/// System table name for triggers
pub const SYS_TRIGGERS: &str = "_sys_triggers";

/// SQL to create the triggers system table
pub const CREATE_TRIGGERS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS _sys_triggers (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    schema TEXT,
    name TEXT NOT NULL,
    table_name TEXT NOT NULL,
    timing TEXT NOT NULL,
    event TEXT NOT NULL,
    for_each_row BOOLEAN NOT NULL,
    language TEXT NOT NULL,
    code TEXT NOT NULL
);
"#;

/// Check if a table name is the triggers system table
pub fn is_triggers_table(_schema: &str, name: &str) -> bool {
    name.eq_ignore_ascii_case(SYS_TRIGGERS)
}

/// Trigger metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTrigger {
    pub id: i64,
    pub schema: Option<String>,
    pub name: String,
    pub table_name: String,
    pub timing: String,
    pub event: String,
    pub for_each_row: bool,
    pub language: String,
    pub code: String,
}
