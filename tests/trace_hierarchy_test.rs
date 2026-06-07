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
fn test_trace_hierarchy() {
    let db = Database::open_in_memory().unwrap();

    let (trace_tx, trace_rx) = crossbeam_channel::bounded(10000);
    let trace_layer = oxibase::common::tracing::SystemTraceLayer::new(trace_tx);

    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    let _ = tracing_subscriber::registry().with(trace_layer).try_init();

    let _shutdown = oxibase::common::tracing::start_trace_flusher(db.engine().clone(), trace_rx);

    db.execute(
        "CREATE TABLE test_hierarchy (id INTEGER PRIMARY KEY, val TEXT)",
        (),
    )
    .unwrap();
    db.execute("INSERT INTO test_hierarchy VALUES (1, 'hello')", ())
        .unwrap();

    let mut rows = db.query("SELECT * FROM test_hierarchy", ()).unwrap();
    assert!(rows.next().is_some());

    std::thread::sleep(Duration::from_millis(500));

    let trace_rows = db
        .query("SELECT name, parent_span_id FROM system.traces", ())
        .unwrap();

    let mut found_db_query = false;
    let mut found_db_execute = false;
    let mut inner_spans_with_parents = 0;

    for row in trace_rows {
        let row = row.unwrap();
        let name: String = row.get(0).unwrap();
        let parent_id: Option<String> = row.get(1).unwrap();

        if name == "db.query" {
            found_db_query = true;
        } else if name == "db.execute" {
            found_db_execute = true;
        } else if (name == "execute_statement" || name == "parse_program") && parent_id.is_some() {
            inner_spans_with_parents += 1;
        }
    }

    assert!(found_db_query, "Expected db.query span");
    assert!(found_db_execute, "Expected db.execute span");
    assert!(
        inner_spans_with_parents > 0,
        "Expected inner executor/parser spans to have a parent_span_id set"
    );
}
