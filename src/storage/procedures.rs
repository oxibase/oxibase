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

//! Stored procedures persistence
//!
//! This module provides storage for stored procedures in system tables.
//! Procedures are stored in the `system.procedures` system table and loaded
//! during database startup.

use serde::{Deserialize, Serialize};

/// System table name for stored procedures
pub const SYS_PROCEDURES: &str = "system.procedures";

/// SQL to create the procedures system table
pub const CREATE_PROCEDURES_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS system.procedures (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    schema TEXT,
    name TEXT NOT NULL,
    parameters TEXT NOT NULL,
    language TEXT NOT NULL,
    code TEXT NOT NULL,
    UNIQUE(schema, name)
);
"#;

/// Check if a table name is the procedures system table
pub fn is_procedures_table(_schema: &str, name: &str) -> bool {
    name.eq_ignore_ascii_case(SYS_PROCEDURES)
}

/// Parameter representation for stored procedures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredProcedureParameter {
    pub mode: String, // We use String here to avoid depending on ast::ParameterMode directly for serialization if we want to decouple, but we can also use an enum
    pub name: String,
    pub data_type: String,
}

/// Stored procedure metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredProcedure {
    pub id: i64,
    pub schema: Option<String>,
    pub name: String,
    pub parameters: Vec<StoredProcedureParameter>,
    pub language: String,
    pub code: String,
}
