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

use oxibase::api::database::Database;

#[test]
fn test_public_schema_fallback() {
    let db = Database::open_in_memory().unwrap();

    // Test 1: Procedure
    db.execute(
        "CREATE PROCEDURE my_proc() LANGUAGE sql AS $$ SELECT 1; $$",
        (),
    )
    .unwrap();
    let result = db
        .query(
            "SELECT schema FROM system.procedures WHERE name = 'MY_PROC'",
            (),
        )
        .unwrap();
    let rows = result.collect_vec().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String>(0).unwrap(), "PUBLIC");

    // Test 2: Function
    db.execute(
        "CREATE FUNCTION my_func() RETURNS INT LANGUAGE sql AS 'SELECT 1;'",
        (),
    )
    .unwrap();
    let result = db
        .query(
            "SELECT schema FROM system.functions WHERE name = 'MY_FUNC'",
            (),
        )
        .unwrap();
    let rows = result.collect_vec().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String>(0).unwrap(), "PUBLIC");

    // Test 3: Table & Trigger
    db.execute("CREATE TABLE users (id INT)", ()).unwrap();
    db.execute(
        "CREATE TRIGGER my_trig BEFORE INSERT ON users FOR EACH ROW LANGUAGE sql AS 'SELECT 1;'",
        (),
    )
    .unwrap();
    let result = db
        .query(
            "SELECT schema FROM system.triggers WHERE name = 'MY_TRIG'",
            (),
        )
        .unwrap();
    let rows = result.collect_vec().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String>(0).unwrap(), "PUBLIC");

    // Test 4: Sequence
    db.execute("CREATE SEQUENCE my_seq", ()).unwrap();
    let result = db.query("SELECT NEXTVAL('my_seq')", ()).unwrap();
    let rows = result.collect_vec().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<i64>(0).unwrap(), 1);

    // Test 5: View
    db.execute("CREATE VIEW my_view AS SELECT 1 AS id", ())
        .unwrap();
    let result = db.query("SELECT id FROM my_view", ()).unwrap();
    let rows = result.collect_vec().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<i64>(0).unwrap(), 1);
}
