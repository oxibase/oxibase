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

//! Internal Traces persistence
//!
//! This module provides storage configuration for internal OpenTelemetry traces
//! in system tables. Traces are stored in the `system.traces` system table.

/// System table name for internal traces
pub const SYS_TRACES: &str = "system.traces";

/// SQL to create the traces system table
pub const CREATE_TRACES_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS system.traces (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    trace_id TEXT NOT NULL,
    span_id TEXT NOT NULL,
    parent_span_id TEXT,
    name TEXT NOT NULL,
    span_kind TEXT NOT NULL,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP NOT NULL,
    duration_ms FLOAT NOT NULL,
    status_code TEXT NOT NULL,
    status_message TEXT,
    attributes TEXT,
    events TEXT
);
"#;

/// Check if a table name is the traces system table
pub fn is_traces_table(schema: &str, name: &str) -> bool {
    schema.eq_ignore_ascii_case("system") && name.eq_ignore_ascii_case("traces")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_traces_table() {
        assert!(is_traces_table("system", "traces"));
        assert!(is_traces_table("SYSTEM", "TRACES"));
        assert!(!is_traces_table("public", "traces"));
        assert!(!is_traces_table("system", "tables"));
    }
}
