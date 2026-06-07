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

use oxibase::api::Database;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::tempdir;

/// Helper to get total size of files in a directory
fn get_dir_size(dir: &Path) -> u64 {
    if !dir.exists() {
        return 0;
    }
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum()
        })
        .unwrap_or(0)
}

#[test]
fn test_telemetry_bypasses_wal() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_path_buf();
    let dsn = format!("file://{}", db_path.display());

    let db = Database::open(&dsn).unwrap();
    let wal_dir = db_path.join("wal");

    // Start trace flusher
    let (trace_tx, trace_rx) = crossbeam_channel::bounded(10000);
    let trace_layer = oxibase::common::tracing::SystemTraceLayer::new(trace_tx);

    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    let _ = tracing_subscriber::registry().with(trace_layer).try_init();

    let _shutdown = oxibase::common::tracing::start_trace_flusher(db.engine().clone(), trace_rx);

    // Initial WAL size (might be empty or have marker)
    let initial_wal_size = get_dir_size(&wal_dir);

    // Create a regular table and insert to verify WAL works
    db.execute(
        "CREATE TABLE test_data (id INTEGER PRIMARY KEY, val TEXT)",
        (),
    )
    .unwrap();
    db.execute("INSERT INTO test_data VALUES (1, 'hello')", ())
        .unwrap();

    // Check WAL size increased for regular data
    std::thread::sleep(Duration::from_millis(100)); // allow flush
    let user_data_wal_size = get_dir_size(&wal_dir);
    assert!(
        user_data_wal_size > initial_wal_size,
        "WAL should increase for user data"
    );

    // Now, emit tracing spans by performing a query
    let mut rows = db.query("SELECT * FROM test_data", ()).unwrap();
    assert!(rows.next().is_some());

    // Allow time for the telemetry flusher to insert the spans into system.traces
    std::thread::sleep(Duration::from_millis(500));

    let final_wal_size = get_dir_size(&wal_dir);

    // Query system.traces to ensure telemetry was actually inserted
    let trace_rows = db.query("SELECT COUNT(*) FROM system.traces", ()).unwrap();

    if let Some(row) = trace_rows.into_iter().next() {
        let count: i64 = row.unwrap().get(0).unwrap();
        assert!(count > 0, "Traces should be inserted into system.traces");
    } else {
        panic!("Failed to query system.traces");
    }

    // Verify WAL size hasn't grown because of telemetry
    assert_eq!(
        final_wal_size, user_data_wal_size,
        "WAL size should not increase due to telemetry insertions"
    );
}
