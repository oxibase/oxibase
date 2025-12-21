// Copyright 2025 Stoolap Contributors
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

//! User-defined functions persistence
//!
//! This module provides storage for user-defined functions in system tables.
//! Functions are stored in the `_sys_functions` system table and loaded
//! during database startup.

use serde::{Deserialize, Serialize};

/// System table name for user-defined functions
pub const SYS_FUNCTIONS: &str = "_sys_functions";

/// SQL to create the functions system table
pub const CREATE_FUNCTIONS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS _sys_functions (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT NOT NULL UNIQUE,
    parameters JSON NOT NULL,
    return_type TEXT NOT NULL,
    language TEXT NOT NULL,
    code TEXT NOT NULL
)
"#;

/// Check if a table name is a system functions table
pub fn is_functions_table(table_name: &str) -> bool {
    table_name.eq_ignore_ascii_case(SYS_FUNCTIONS)
}

/// Simplified parameter representation for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredParameter {
    pub name: String,
    pub data_type: String,
}

/// Stored function metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFunction {
    pub id: i64,
    pub name: String,
    pub parameters: Vec<StoredParameter>, // Simplified parameter representation
    pub return_type: String,
    pub language: String,
    pub code: String,
}

