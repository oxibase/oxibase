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

//! Internal Logging persistence
//!
//! This module provides storage configuration for internal logs in system tables.
//! Logs are stored in the `system.logs` system table.

/// System table name for internal logs
pub const SYS_LOGS: &str = "system.logs";

/// SQL to create the logs system table
pub const CREATE_LOGS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS system.logs (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    level TEXT NOT NULL,
    target TEXT NOT NULL,
    message TEXT NOT NULL,
    json_fields TEXT,
    trace_id TEXT,
    span_id TEXT
);
"#;

/// Check if a table name is the logs system table
pub fn is_logs_table(schema: &str, name: &str) -> bool {
    schema.eq_ignore_ascii_case("system") && name.eq_ignore_ascii_case("logs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_logs_table() {
        assert!(is_logs_table("system", "logs"));
        assert!(is_logs_table("SYSTEM", "LOGS"));
        assert!(!is_logs_table("public", "logs"));
        assert!(!is_logs_table("system", "tables"));
    }
}
