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
use std::time::Duration;

#[test]
fn test_tracing_ingestion() {
    let db = Database::open_in_memory().unwrap();

    // Start the trace flusher (simulating what oxibase.rs does)
    let (trace_tx, trace_rx) = crossbeam_channel::bounded(10000);
    let trace_layer = oxibase::common::tracing::SystemTraceLayer::new(trace_tx);

    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    // We try to init, but it might fail if another test initialized it already.
    let _ = tracing_subscriber::registry().with(trace_layer).try_init();

    let _shutdown = oxibase::common::tracing::start_trace_flusher(db.engine().clone(), trace_rx);

    // Create a table and insert a row
    db.execute(
        "CREATE TABLE test_traces (id INTEGER PRIMARY KEY, val TEXT)",
        (),
    )
    .unwrap();
    db.execute("INSERT INTO test_traces VALUES (1, 'hello')", ())
        .unwrap();

    // Select to trigger the parser, planner, executor
    let mut rows = db.query("SELECT * FROM test_traces", ()).unwrap();
    assert!(rows.next().is_some());

    // Allow time for the flusher to insert the spans into system.traces
    std::thread::sleep(Duration::from_millis(500));

    // Query system.traces
    let trace_rows = db
        .query("SELECT name, duration_ms FROM system.traces", ())
        .unwrap();

    let mut found_spans = std::collections::HashSet::new();
    for row in trace_rows {
        let row = row.unwrap();
        let name: String = row.get(0).unwrap();
        found_spans.insert(name);
    }

    // We should see spans for execute_statement, choose_access_method, choose_join_algorithm, parse_program
    // depending on what was actually hit.
    assert!(
        !found_spans.is_empty(),
        "Expected to find some trace spans in system.traces"
    );
}
