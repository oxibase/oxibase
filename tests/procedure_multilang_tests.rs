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

use oxibase::api::Database;
use oxibase::core::Value;

#[test]
#[cfg(feature = "js")]
fn test_js_procedure() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE PROCEDURE multiply_js(a INT, b INT, OUT res INT)
        LANGUAGE js
        AS '
            res = a * b;
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(
        res.is_ok(),
        "Failed to create js procedure: {:?}",
        res.err()
    );

    let call_sql = "CALL multiply_js(5, 4, 0);";
    let res = db.query(call_sql, ());
    assert!(res.is_ok(), "Failed to call js procedure: {:?}", res.err());

    let mut results = res.unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_int64().unwrap(), 20);
}

#[test]
#[cfg(feature = "python")]
fn test_python_procedure() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE PROCEDURE concat_py(a TEXT, b TEXT, OUT res TEXT)
        LANGUAGE python
        AS '
res = a + " " + b
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(
        res.is_ok(),
        "Failed to create python procedure: {:?}",
        res.err()
    );

    let call_sql = "CALL concat_py('hello', 'world', '');";
    let res = db.query(call_sql, ());
    assert!(
        res.is_ok(),
        "Failed to call python procedure: {:?}",
        res.err()
    );

    let mut results = res.unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(
        row.get::<Value>(0).unwrap().as_str().unwrap(),
        "hello world"
    );
}

#[test]
#[cfg(feature = "js")]
fn test_js_sql_execution() {
    let db = Database::open_in_memory().unwrap();
    db.execute(
        "CREATE TABLE js_logs(id INTEGER PRIMARY KEY AUTO_INCREMENT, msg TEXT);",
        (),
    )
    .unwrap();

    let create_sql = r#"
        CREATE PROCEDURE log_js(msg TEXT)
        LANGUAGE js
        AS '
            oxibase.execute("INSERT INTO js_logs(msg) VALUES (''Hello JS'')");
        ';
    "#;

    db.execute(create_sql, ()).unwrap();
    db.execute("CALL log_js('Hello JS');", ()).unwrap();

    let mut results = db.query("SELECT msg FROM js_logs;", ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_str().unwrap(), "Hello JS");
}

#[test]
#[cfg(feature = "python")]
fn test_python_sql_execution() {
    let db = Database::open_in_memory().unwrap();
    db.execute(
        "CREATE TABLE py_logs(id INTEGER PRIMARY KEY AUTO_INCREMENT, msg TEXT);",
        (),
    )
    .unwrap();

    let create_sql = r#"
        CREATE PROCEDURE log_py(msg TEXT)
        LANGUAGE python
        AS '
import oxibase
oxibase.execute("INSERT INTO py_logs(msg) VALUES (''Hello Python'')")
        ';
    "#;

    db.execute(create_sql, ()).unwrap();
    db.execute("CALL log_py('Hello Python');", ()).unwrap();

    let mut results = db.query("SELECT msg FROM py_logs;", ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(
        row.get::<Value>(0).unwrap().as_str().unwrap(),
        "Hello Python"
    );
}

#[test]
#[cfg(feature = "js")]
fn test_js_transaction_commit_rollback() {
    let db = Database::open_in_memory().unwrap();
    db.execute(
        "CREATE TABLE tx_test_js(id INTEGER PRIMARY KEY, val TEXT);",
        (),
    )
    .unwrap();

    let create_sql = r#"
        CREATE PROCEDURE tx_proc_js() 
        LANGUAGE js 
        AS '
            // Insert and commit
            oxibase.execute("INSERT INTO tx_test_js(id, val) VALUES (1, ''first'')");
            commit();
            
            // Insert and rollback
            oxibase.execute("INSERT INTO tx_test_js(id, val) VALUES (2, ''second'')");
            rollback();
            
            // Insert after rollback and commit
            oxibase.execute("INSERT INTO tx_test_js(id, val) VALUES (3, ''third'')");
            commit();
        ';
    "#;

    db.execute(create_sql, ()).unwrap();
    db.execute("CALL tx_proc_js();", ()).unwrap();

    let mut results = db
        .query("SELECT id, val FROM tx_test_js ORDER BY id;", ())
        .unwrap();

    // First row should exist
    let row1 = results.next().unwrap().unwrap();
    assert_eq!(row1.get::<Value>(0).unwrap().as_int64().unwrap(), 1);
    assert_eq!(row1.get::<Value>(1).unwrap().as_str().unwrap(), "first");

    // Third row should exist (second was rolled back)
    let row3 = results.next().unwrap().unwrap();
    assert_eq!(row3.get::<Value>(0).unwrap().as_int64().unwrap(), 3);
    assert_eq!(row3.get::<Value>(1).unwrap().as_str().unwrap(), "third");

    // No more rows
    assert!(results.next().is_none());
}

#[test]
#[cfg(feature = "python")]
fn test_python_transaction_commit_rollback() {
    let db = Database::open_in_memory().unwrap();
    db.execute(
        "CREATE TABLE tx_test_py(id INTEGER PRIMARY KEY, val TEXT);",
        (),
    )
    .unwrap();

    let create_sql = r#"
        CREATE PROCEDURE tx_proc_py() 
        LANGUAGE python 
        AS '
import oxibase
# Insert and commit
oxibase.execute("INSERT INTO tx_test_py(id, val) VALUES (1, ''first'')")
oxibase.commit()

# Insert and rollback
oxibase.execute("INSERT INTO tx_test_py(id, val) VALUES (2, ''second'')")
oxibase.rollback()

# Insert after rollback and commit
oxibase.execute("INSERT INTO tx_test_py(id, val) VALUES (3, ''third'')")
oxibase.commit()
        ';
    "#;

    db.execute(create_sql, ()).unwrap();
    db.execute("CALL tx_proc_py();", ()).unwrap();

    let mut results = db
        .query("SELECT id, val FROM tx_test_py ORDER BY id;", ())
        .unwrap();

    // First row should exist
    let row1 = results.next().unwrap().unwrap();
    assert_eq!(row1.get::<Value>(0).unwrap().as_int64().unwrap(), 1);
    assert_eq!(row1.get::<Value>(1).unwrap().as_str().unwrap(), "first");

    // Third row should exist (second was rolled back)
    let row3 = results.next().unwrap().unwrap();
    assert_eq!(row3.get::<Value>(0).unwrap().as_int64().unwrap(), 3);
    assert_eq!(row3.get::<Value>(1).unwrap().as_str().unwrap(), "third");

    // No more rows
    assert!(results.next().is_none());
}
