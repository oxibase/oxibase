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
use oxibase::Database;

#[test]
fn test_system_transactions() {
    let db = Database::open("memory://").unwrap();

    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY)", ())
        .unwrap();

    // Start a transaction
    db.execute("BEGIN", ()).unwrap();

    // Insert a row to keep transaction active
    db.execute("INSERT INTO users (id) VALUES (1)", ()).unwrap();

    // Query active transactions
    let result = db
        .query("SELECT state FROM system.transactions", ())
        .unwrap();
    let rows = result.collect_vec().unwrap();

    assert!(
        !rows.is_empty(),
        "Should have at least one active transaction"
    );

    let state: String = rows[0].get(0).unwrap();
    assert!(state.contains("ACTIVE"));

    db.execute("COMMIT", ()).unwrap();

    // Once committed, it shouldn't show up
    let result2 = db
        .query("SELECT state FROM system.transactions", ())
        .unwrap();
    let rows2 = result2.collect_vec().unwrap();
    assert!(
        rows2.is_empty()
            || !rows2
                .iter()
                .any(|r| r.get::<String>(0).unwrap().contains("ACTIVE (1)"))
    );
}
