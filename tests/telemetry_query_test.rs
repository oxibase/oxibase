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
fn test_telemetry_query() {
    let db = Database::open_in_memory().unwrap();

    let (log_tx, log_rx) = crossbeam_channel::bounded(100);
    let log_layer = oxibase::common::logging::InternalLogLayer::new(log_tx);

    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    let _ = tracing_subscriber::registry().with(log_layer).try_init();

    let _shutdown = oxibase::common::logging::start_log_flusher(db.engine().clone(), log_rx);

    // Emit some logs
    tracing::error!(target: "test_target", "This is an error log");
    tracing::warn!(target: "test_target", "This is a warning log");
    tracing::info!(target: "test_target", "This is an info log");

    // Allow time for flusher
    std::thread::sleep(Duration::from_millis(500));

    // Query 1: Basic select
    let rows = db
        .query(
            "SELECT level, message FROM system.logs WHERE target = 'test_target'",
            (),
        )
        .unwrap();
    let mut count = 0;
    for _ in rows {
        count += 1;
    }
    assert_eq!(count, 3, "Should have 3 test_target logs");

    // Query 2: WHERE clause
    let rows = db
        .query(
            "SELECT level FROM system.logs WHERE target = 'test_target' AND level = 'ERROR'",
            (),
        )
        .unwrap();
    let mut err_count = 0;
    for _ in rows {
        err_count += 1;
    }
    assert_eq!(err_count, 1, "Should filter ERROR logs");

    // Query 3: ORDER BY and LIMIT
    let rows = db.query("SELECT level FROM system.logs WHERE target = 'test_target' ORDER BY level DESC LIMIT 2", ()).unwrap();
    let mut levels = Vec::new();
    for row in rows {
        let level: String = row.unwrap().get(0).unwrap();
        levels.push(level);
    }
    assert_eq!(levels.len(), 2);
    // String DESC sort: WARN, INFO, ERROR.
    // So top 2 should be WARN and INFO.
    assert_eq!(levels[0], "WARN");
    assert_eq!(levels[1], "INFO");
}
