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
fn test_system_tables() {
    let db = Database::open("memory://").unwrap();

    // Create a test table
    db.execute(
        "CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT)",
        (),
    )
    .unwrap();

    // Query system.tables
    let result = db
        .query("SELECT table_name FROM system.tables", ())
        .unwrap();
    let rows = result.collect_vec().unwrap();

    // Convert to simple strings for verification
    let mut table_names = vec![];
    for row in rows {
        table_names.push(row.get::<String>(0).unwrap());
    }

    assert!(table_names.contains(&"test_table".to_string()));
    // We also expect internal tables like system.functions depending on engine init
}

#[test]
fn test_system_columns() {
    let db = Database::open("memory://").unwrap();

    db.execute(
        "CREATE TABLE user_profiles (id INTEGER PRIMARY KEY, email TEXT, active BOOLEAN)",
        (),
    )
    .unwrap();

    let result = db
        .query(
            "SELECT column_name, data_type FROM system.columns WHERE table_name = 'user_profiles'",
            (),
        )
        .unwrap();
    let rows = result.collect_vec().unwrap();

    assert_eq!(rows.len(), 3);

    // Check specific columns
    let id_row = rows
        .iter()
        .find(|r| r.get::<String>(0).unwrap() == "id")
        .unwrap();
    assert_eq!(id_row.get::<String>(1).unwrap(), "Integer");

    let email_row = rows
        .iter()
        .find(|r| r.get::<String>(0).unwrap() == "email")
        .unwrap();
    assert_eq!(email_row.get::<String>(1).unwrap(), "Text");
}

#[test]
fn test_reserved_namespace_protection() {
    let db = Database::open("memory://").unwrap();

    // These should fail
    let err1 = db
        .execute("CREATE TABLE system.my_table (id INT)", ())
        .unwrap_err();
    assert!(err1
        .to_string()
        .contains("cannot modify reserved namespace"));

    let err2 = db
        .execute("CREATE TABLE information_schema.my_table (id INT)", ())
        .unwrap_err();
    assert!(err2
        .to_string()
        .contains("cannot modify reserved namespace"));

    let err3 = db.execute("DROP TABLE system.tables", ());
    assert!(err3.is_err());

    let err4 = db.execute("INSERT INTO system.tables (id) VALUES (1)", ());
    assert!(err4.is_err());
}
