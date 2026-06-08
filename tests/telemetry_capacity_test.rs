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
use oxibase::core::{DataType, Row, Value};
use oxibase::Engine;

#[test]
fn test_telemetry_capacity() {
    let db = Database::open_in_memory().unwrap();

    let mut tx = db.engine().begin_transaction().unwrap();

    // system.traces gets instantiated as a SystemRingBufferTable with capacity 100_000
    // Let's test it by requesting the table.
    let mut table = tx.get_table("system.traces").unwrap();

    // We'll insert 100,005 rows.
    // However, going through engine creates it with 100,000 capacity hardcoded.
    // Instead of inserting 100,005 rows manually one-by-one which might be slow,
    // let's insert a batch.

    let mut rows = Vec::with_capacity(100_005);
    for i in 0..100_005 {
        // Table system.traces schema: id, trace_id, span_id, parent_span_id, name, span_kind, start_time, end_time, duration_ms, status_code, status_message, attributes, events
        rows.push(Row::from_values(vec![
            Value::Integer(i as i64),
            Value::Text("trace_id".into()),
            Value::Text("span_id".into()),
            Value::Null(DataType::Text),
            Value::Text("name".into()),
            Value::Text("INTERNAL".into()),
            Value::Timestamp(chrono::Utc::now()),
            Value::Timestamp(chrono::Utc::now()),
            Value::Float(1.0),
            Value::Text("OK".into()),
            Value::Null(DataType::Text),
            Value::Text("{}".into()),
            Value::Null(DataType::Text),
        ]));
    }

    // Since insert_batch drops the oldest elements if it exceeds capacity
    table.insert_batch(rows).unwrap();

    // Check row count
    let mut scanner = table.scan(&[0], None).unwrap();
    let mut count = 0;
    while scanner.next() {
        count += 1;
    }

    assert_eq!(
        count, 100_000,
        "Capacity should be strictly bounded to 100,000"
    );

    tx.commit().unwrap();

    // Also check via SQL
    let count_rows = db.query("SELECT COUNT(*) FROM system.traces", ()).unwrap();
    if let Some(row) = count_rows.into_iter().next() {
        let sql_count: i64 = row.unwrap().get(0).unwrap();
        assert_eq!(sql_count, 100_000, "SQL COUNT(*) should match capacity");
    } else {
        panic!("Failed to query system.traces");
    }
}
