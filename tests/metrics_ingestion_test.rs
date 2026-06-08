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
fn test_metrics_ingestion() {
    // 1. Setup channels and tracing layer
    let (metrics_tx, metrics_rx) = crossbeam_channel::bounded(100);
    let metrics_layer = oxibase::common::metrics::SystemMetricsLayer::new(metrics_tx);

    let (trace_tx, trace_rx) = crossbeam_channel::bounded(100);
    let trace_layer = oxibase::common::tracing::SystemTraceLayer::new(trace_tx);

    let subscriber = Registry::default().with(trace_layer).with(metrics_layer);

    // Set the global default subscriber for this test thread
    let _guard = tracing::subscriber::set_default(subscriber);

    // 2. Open an in-memory database
    // This uses the same engine that the flusher thread will use
    let db = Database::open("memory://").expect("Failed to open database");

    // Start the background flushers
    let (shutdown_metrics, handle_metrics) =
        oxibase::common::metrics::start_metrics_flusher(db.engine().clone(), metrics_rx);
    let (shutdown_trace, handle_trace) =
        oxibase::common::tracing::start_trace_flusher(db.engine().clone(), trace_rx);

    // Give the engine a moment to be fully ready
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 3. Emit some metrics manually (as if from code) inside a span
    {
        let span = tracing::info_span!("metric_span");
        let _enter = span.enter();

        tracing::info!(
            target: "oxibase::metrics",
            metric_type = "counter",
            metric_name = "test_queries",
            value = 1.0,
            unit = "count",
            description = "Test query counter"
        );

        tracing::info!(
            target: "oxibase::metrics",
            metric_type = "gauge",
            metric_name = "test_memory",
            value = 1024.0,
            unit = "bytes",
            description = "Test memory gauge"
        );
    }

    // 4. Also run a real query which should trigger queries_total metric
    let _ = db.execute("SELECT 1", ()).expect("Failed to execute query");

    // 5. Wait for the flusher to process
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // 6. Query the system.metrics table
    let result = db
        .query(
            "SELECT name, metric_type, value, attributes FROM system.metrics ORDER BY id",
            (),
        )
        .expect("Failed to query metrics");

    let rows: Vec<_> = result.collect();
    assert!(
        !rows.is_empty(),
        "Expected metrics to be ingested into system.metrics"
    );

    // Verify manually emitted metrics
    let mut found_test_queries = false;
    let mut found_test_memory = false;
    let mut found_queries_total = false;

    for row_res in rows {
        let row = row_res.expect("row error");
        let name: String = row.get(0).expect("name");
        let metric_type: String = row.get(1).expect("metric_type");
        let value: f64 = row.get(2).expect("value");
        let attributes: Option<String> = row.get(3).unwrap_or(None);

        if name == "test_queries" {
            assert_eq!(metric_type, "counter");
            assert_eq!(value, 1.0);

            // Verify trace_id and span_id are in attributes
            let attr_str = attributes.unwrap_or_default();
            assert!(
                attr_str.contains("trace_id"),
                "attributes should contain trace_id"
            );
            assert!(
                attr_str.contains("span_id"),
                "attributes should contain span_id"
            );

            found_test_queries = true;
        } else if name == "test_memory" {
            assert_eq!(metric_type, "gauge");
            assert_eq!(value, 1024.0);
            found_test_memory = true;
        } else if name == "queries_total" {
            assert_eq!(metric_type, "counter");
            // Since we ran SELECT 1 and potentially internal queries like SELECT name, etc
            assert!(value >= 1.0);
            found_queries_total = true;
        }
    }

    assert!(found_test_queries, "Did not find test_queries metric");
    assert!(found_test_memory, "Did not find test_memory metric");
    assert!(found_queries_total, "Did not find queries_total metric");

    // Clean up
    shutdown_metrics.store(true, std::sync::atomic::Ordering::SeqCst);
    shutdown_trace.store(true, std::sync::atomic::Ordering::SeqCst);
    let _ = handle_metrics.join();
    let _ = handle_trace.join();
}
