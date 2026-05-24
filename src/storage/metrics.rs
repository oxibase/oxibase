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

//! Internal Metrics persistence
//!
//! This module provides storage configuration for internal OpenTelemetry metrics
//! in system tables. Metrics are stored in the `system.metrics` system table.

/// System table name for internal metrics
pub const SYS_METRICS: &str = "system.metrics";

/// SQL to create the metrics system table
pub const CREATE_METRICS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS system.metrics (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    unit TEXT,
    metric_type TEXT NOT NULL,
    value FLOAT NOT NULL,
    attributes TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
"#;

/// Check if a table name is the metrics system table
pub fn is_metrics_table(schema: &str, name: &str) -> bool {
    schema.eq_ignore_ascii_case("system") && name.eq_ignore_ascii_case("metrics")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_metrics_table() {
        assert!(is_metrics_table("system", "metrics"));
        assert!(is_metrics_table("SYSTEM", "METRICS"));
        assert!(!is_metrics_table("public", "metrics"));
        assert!(!is_metrics_table("system", "tables"));
    }
}
