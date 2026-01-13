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

//! Stored functions persistence
//!
//! This module provides storage for stored functions in system tables.
//! Stored functions are scripts in Rhai, Python, or JavaScript that can be
//! executed and can call arbitrary SQL.

use serde::{Deserialize, Serialize};

/// System table name for stored functions
pub const STORED_FUNCTIONS: &str = "stored_functions";

/// SQL to create the stored functions system table
pub const CREATE_STORED_FUNCTIONS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS stored_functions (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT NOT NULL,
    language TEXT NOT NULL CHECK (language IN ('rhai', 'python', 'javascript')),
    code TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name)
);

CREATE INDEX IF NOT EXISTS idx_stored_functions_language ON stored_functions (language);
CREATE INDEX IF NOT EXISTS idx_stored_functions_name ON stored_functions (name);
"#;

/// Check if a table name is a stored functions table
pub fn is_stored_functions_table(table_name: &str) -> bool {
    table_name.eq_ignore_ascii_case(STORED_FUNCTIONS)
}

/// Stored function metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFunction {
    pub id: i64,
    pub name: String,
    pub language: String,
    pub code: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
