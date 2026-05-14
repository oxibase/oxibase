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

//! Job Scheduler persistence
//!
//! This module provides storage configuration for job schedules in system tables.
//! Schedules are stored in the `system.cron` system table.

/// System table name for scheduled jobs
pub const SYS_CRON: &str = "system.cron";

/// System table name for job execution logs
pub const SYS_CRON_RUNS: &str = "system.cron_runs";

/// SQL to create the cron system table
pub const CREATE_CRON_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS system.cron (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT UNIQUE NOT NULL,
    schedule TEXT NOT NULL,
    command TEXT NOT NULL,
    active BOOLEAN DEFAULT true
);
"#;

/// SQL to create the cron execution logs system table
pub const CREATE_CRON_RUNS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS system.cron_runs (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    job_id INTEGER NOT NULL,
    status TEXT NOT NULL,
    return_message TEXT,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP
);
"#;

/// Check if a table name is the cron system table
pub fn is_cron_table(schema: &str, name: &str) -> bool {
    schema.eq_ignore_ascii_case("system") && name.eq_ignore_ascii_case("cron")
}

/// Check if a table name is the cron runs system table
pub fn is_cron_runs_table(schema: &str, name: &str) -> bool {
    schema.eq_ignore_ascii_case("system") && name.eq_ignore_ascii_case("cron_runs")
}
