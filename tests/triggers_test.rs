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

use oxibase::core::Value;
use oxibase::executor::Executor;
use oxibase::storage::mvcc::engine::MVCCEngine;
use std::sync::Arc;

fn setup_executor() -> Executor {
    let engine = MVCCEngine::in_memory();
    engine.open_engine().unwrap();
    Executor::new(Arc::new(engine))
}

#[test]
fn test_create_and_drop_trigger() {
    let executor = setup_executor();
    executor
        .execute("CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT)")
        .unwrap();

    // Create trigger
    let result = executor.execute(
        r#"
        CREATE TRIGGER test_trigger
        BEFORE INSERT ON test_table
        FOR EACH ROW
        LANGUAGE rhai
        AS '
            if NEW.id < 0 {
                throw "Negative ID not allowed";
            }
        ';
    "#,
    );

    assert!(
        result.is_ok(),
        "Failed to create trigger: {:?}",
        result.err()
    );

    // Test validation
    let insert_err = executor.execute("INSERT INTO test_table (id, name) VALUES (-1, 'test')");
    assert!(insert_err.is_err());
    if let Err(e) = insert_err {
        assert!(e.to_string().contains("Negative ID not allowed"));
    }

    // Test valid insert
    let insert_ok = executor.execute("INSERT INTO test_table (id, name) VALUES (1, 'test')");
    assert!(insert_ok.is_ok());

    // Drop trigger
    let drop_result = executor.execute("DROP TRIGGER test_trigger ON test_table");
    assert!(drop_result.is_ok());
}

#[test]
fn test_data_transformation_trigger() {
    let executor = setup_executor();
    executor
        .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
        .unwrap();

    executor
        .execute(
            r#"
        CREATE TRIGGER normalize_name
        BEFORE INSERT ON users
        FOR EACH ROW
        LANGUAGE rhai
        AS '
            NEW.name = "PREFIX_" + NEW.name;
        ';
    "#,
        )
        .unwrap();

    executor
        .execute("INSERT INTO users (id, name) VALUES (1, 'alice')")
        .unwrap();

    let mut result = executor
        .execute("SELECT name FROM users WHERE id = 1")
        .unwrap();
    assert!(result.next());
    assert_eq!(result.row().get(0), Some(&Value::text("PREFIX_alice")));
}

#[test]
fn test_audit_trigger() {
    let executor = setup_executor();
    executor
        .execute("CREATE TABLE products (id INTEGER PRIMARY KEY, price FLOAT)")
        .unwrap();
    executor
        .execute("CREATE TABLE audit_log (product_id INTEGER, old_price FLOAT, new_price FLOAT)")
        .unwrap();

    executor.execute(r#"
        CREATE TRIGGER audit_price
        AFTER UPDATE ON products
        FOR EACH ROW
        LANGUAGE rhai
        AS '
            if OLD.price != NEW.price {
                oxibase::execute("INSERT INTO audit_log (product_id, old_price, new_price) VALUES (" + OLD.id + ", " + OLD.price + ", " + NEW.price + ")");
            }
        ';
    "#).unwrap();

    executor
        .execute("INSERT INTO products (id, price) VALUES (1, 10.0)")
        .unwrap();

    // Update price
    executor
        .execute("UPDATE products SET price = 15.0 WHERE id = 1")
        .unwrap();

    // Check audit log
    let mut result = executor
        .execute("SELECT old_price, new_price FROM audit_log WHERE product_id = 1")
        .unwrap();
    assert!(result.next());
    let row = result.row();
    assert_eq!(row.get(0), Some(&Value::Float(10.0)));
    assert_eq!(row.get(1), Some(&Value::Float(15.0)));
}

#[test]
fn test_python_trigger() {
    let executor = setup_executor();
    executor
        .execute("CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance FLOAT)")
        .unwrap();

    // Create Python trigger
    let result = executor.execute(
        r#"
        CREATE TRIGGER test_py_trigger
        BEFORE INSERT ON accounts
        FOR EACH ROW
        LANGUAGE python
        AS '
if NEW["balance"] < 0:
    raise RuntimeError("Negative balance not allowed")
'
    "#,
    );

    assert!(
        result.is_ok(),
        "Failed to create trigger: {:?}",
        result.err()
    );

    // Test validation
    let insert_err = executor.execute("INSERT INTO accounts (id, balance) VALUES (1, -50.0)");
    assert!(insert_err.is_err());
    if let Err(e) = insert_err {
        assert!(e.to_string().contains("Negative balance not allowed"));
    }

    // Test valid insert
    let insert_ok = executor.execute("INSERT INTO accounts (id, balance) VALUES (2, 100.0)");
    assert!(insert_ok.is_ok());

    // Test transformation trigger
    executor
        .execute(
            r#"
        CREATE TRIGGER test_py_transform
        BEFORE UPDATE ON accounts
        FOR EACH ROW
        LANGUAGE python
        AS '
NEW["balance"] = NEW["balance"] + 10.0
'
    "#,
        )
        .unwrap();

    executor
        .execute("UPDATE accounts SET balance = 100.0 WHERE id = 2")
        .unwrap();
    let mut result = executor
        .execute("SELECT balance FROM accounts WHERE id = 2")
        .unwrap();
    assert!(result.next());
    assert_eq!(result.row().get(0), Some(&Value::Float(110.0)));
}

#[test]
fn test_js_trigger() {
    let executor = setup_executor();
    executor
        .execute("CREATE TABLE accounts_js (id INTEGER PRIMARY KEY, balance FLOAT)")
        .unwrap();
    executor
        .execute("CREATE TABLE audit_js (id INTEGER, amount FLOAT)")
        .unwrap();

    // Create JS trigger
    let result = executor.execute(
        r#"
        CREATE TRIGGER test_js_trigger
        AFTER UPDATE ON accounts_js
        FOR EACH ROW
        LANGUAGE js
        AS '
if (OLD.balance !== NEW.balance) {
    let diff = NEW.balance - OLD.balance;
    oxibase.execute("INSERT INTO audit_js (id, amount) VALUES (" + NEW.id + ", " + diff + ")");
}
'
    "#,
    );

    assert!(
        result.is_ok(),
        "Failed to create trigger: {:?}",
        result.err()
    );

    // Insert initial
    executor
        .execute("INSERT INTO accounts_js (id, balance) VALUES (1, 100.0)")
        .unwrap();

    // Update balance
    let update_ok = executor.execute("UPDATE accounts_js SET balance = 150.0 WHERE id = 1");
    if let Err(e) = &update_ok {
        println!("Update failed: {:?}", e);
    }
    assert!(update_ok.is_ok());

    // Check audit log
    let mut result = executor
        .execute("SELECT amount FROM audit_js WHERE id = 1")
        .unwrap();
    assert!(result.next());
    assert_eq!(result.row().get(0), Some(&Value::Float(50.0)));
}
