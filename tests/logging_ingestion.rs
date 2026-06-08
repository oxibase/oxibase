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
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

#[test]
fn test_logging_ingestion_with_correlation() {
    let (log_tx, log_rx) = crossbeam_channel::bounded(100);
    let log_layer = oxibase::common::logging::InternalLogLayer::new(log_tx);

    let (trace_tx, trace_rx) = crossbeam_channel::bounded(100);
    let trace_layer = oxibase::common::tracing::SystemTraceLayer::new(trace_tx);

    let subscriber = Registry::default().with(trace_layer).with(log_layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let db = Database::open("memory://").expect("Failed to open database");

    let (shutdown_log, handle_log) =
        oxibase::common::logging::start_log_flusher(db.engine().clone(), log_rx);
    let (shutdown_trace, handle_trace) =
        oxibase::common::tracing::start_trace_flusher(db.engine().clone(), trace_rx);

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Emit log inside span
    {
        let span = tracing::info_span!("test_operation");
        let _enter = span.enter();
        tracing::error!(code = 500, user = "test_user", "A test error occurred");
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    let result = db
        .query(
            "SELECT message, trace_id, span_id, json_fields FROM system.logs",
            (),
        )
        .expect("Failed to query logs");

    let rows: Vec<_> = result.collect();
    assert!(!rows.is_empty(), "Expected logs to be ingested");

    let mut found = false;
    for row_res in rows {
        let row = row_res.unwrap();
        let message: String = row.get(0).unwrap();
        let trace_id: Option<String> = row.get(1).unwrap();
        let span_id: Option<String> = row.get(2).unwrap();
        let json_fields: Option<String> = row.get(3).unwrap();

        if message == "A test error occurred" {
            found = true;
            assert!(trace_id.is_some(), "trace_id should be populated");
            assert!(span_id.is_some(), "span_id should be populated");

            let json_str = json_fields.unwrap();
            assert!(json_str.contains("500"), "json_fields should contain code");
            assert!(
                json_str.contains("test_user"),
                "json_fields should contain user"
            );
        }
    }

    assert!(found, "Did not find expected log entry");

    shutdown_log.store(true, std::sync::atomic::Ordering::SeqCst);
    shutdown_trace.store(true, std::sync::atomic::Ordering::SeqCst);
    let _ = handle_log.join();
    let _ = handle_trace.join();
}
