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

//! Tests based on the Stored Procedures tutorial

use oxibase::api::Database;
use oxibase::core::Value;

#[test]
fn test_tutorial_procedures() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setting Up the Environment
    let db = Database::open_in_memory()?;

    db.execute(
        "CREATE TABLE accounts (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            balance FLOAT NOT NULL
        )",
        (),
    )?;

    db.execute(
        "CREATE TABLE transaction_log (
            id INTEGER PRIMARY KEY AUTO_INCREMENT,
            account_id INTEGER,
            amount FLOAT,
            type TEXT,
            timestamp TEXT
        )",
        (),
    )?;

    db.execute("INSERT INTO accounts VALUES (1, 'Alice', 1000.0)", ())?;
    db.execute("INSERT INTO accounts VALUES (2, 'Bob', 500.0)", ())?;

    // 2. Creating Your First Procedure
    // Note: 'now()' isn't available in Rhai by default, so we'll use a fixed string for the test
    db.execute(
         r#"CREATE PROCEDURE log_transaction(acc_id INTEGER, amt FLOAT, trans_type TEXT)
         LANGUAGE rhai AS '
             let timestamp = "2023-01-01T12:00:00Z";
             execute(`INSERT INTO transaction_log (account_id, amount, type, timestamp) VALUES (${acc_id}, ${amt}, ''${trans_type}'', ''${timestamp}'')`);
         '"#,
         (),
     )?;

    // 3. Implementing Business Logic
    db.execute(
        r#"CREATE PROCEDURE transfer(from_id INTEGER, to_id INTEGER, amount FLOAT)
        LANGUAGE rhai AS '
            // 1. Validation
            let src_rows = execute(`SELECT balance FROM accounts WHERE id = ${from_id}`);
            if src_rows.len() == 0 {
                throw "Source account not found";
            }

            let dest_rows = execute(`SELECT id FROM accounts WHERE id = ${to_id}`);
            if dest_rows.len() == 0 {
                throw "Destination account not found";
            }

            let balance = src_rows[0].balance;
            if balance < amount {
                throw `Insufficient funds: Balance is ${balance}, required ${amount}`;
            }

            // 2. Perform Transfer
            execute(`UPDATE accounts SET balance = balance - ${amount} WHERE id = ${from_id}`);
            execute(`UPDATE accounts SET balance = balance + ${amount} WHERE id = ${to_id}`);

            // 3. Log Transaction
            execute(`CALL log_transaction(${from_id}, ${amount}, ''debit'')`);
            execute(`CALL log_transaction(${to_id}, ${amount}, ''credit'')`);
        '"#,
        (),
    )?;

    // 4. Executing Procedures

    // Check initial balances
    let alice_balance: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 1", ())?;
    assert_eq!(alice_balance, 1000.0);

    let bob_balance: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 2", ())?;
    assert_eq!(bob_balance, 500.0);

    // Execute the transfer
    db.execute("CALL transfer(1, 2, 200.0)", ())?;

    // Check balances after transfer
    let alice_balance: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 1", ())?;
    assert_eq!(alice_balance, 800.0); // 1000 - 200

    let bob_balance: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 2", ())?;
    assert_eq!(bob_balance, 700.0); // 500 + 200

    // Check the log
    let log_count: i64 = db.query_one("SELECT COUNT(*) FROM transaction_log", ())?;
    assert_eq!(log_count, 2);

    // 5. Error Handling and Transactions

    // Test insufficient funds error
    let result = db.execute("CALL transfer(1, 2, 5000.0)", ());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Insufficient funds"));

    // Verify balances haven't changed
    let alice_balance: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 1", ())?;
    assert_eq!(alice_balance, 800.0); // Unchanged

    // Transaction rollback test
    // Note: Our API doesn't support nested transactions or explicit BEGIN/COMMIT in SQL string execution in the same way
    // for this test harness, but we can verify the atomicity of the CALL itself.
    // The CALL statement executes in its own transaction (or part of existing one).
    // When it fails, all changes within it (UPDATEs) are rolled back.

    // Verify logs count is still 2 (failed transfer added no logs)
    let log_count: i64 = db.query_one("SELECT COUNT(*) FROM transaction_log", ())?;
    assert_eq!(log_count, 2);

    Ok(())
}

#[test]
fn test_call_in_transaction() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("memory://")?;

    // Create accounts table
    db.execute(
        "CREATE TABLE accounts (id INTEGER PRIMARY KEY, name TEXT, balance FLOAT)",
        (),
    )?;
    db.execute(
        "INSERT INTO accounts VALUES (1, 'Alice', 1000.0), (2, 'Bob', 500.0)",
        (),
    )?;

    // Create transfer procedure
    db.execute(
        r#"CREATE PROCEDURE transfer(from_id INTEGER, to_id INTEGER, amount FLOAT)
        LANGUAGE rhai AS '
            let src_rows = execute(`SELECT balance FROM accounts WHERE id = ${from_id}`);
            if src_rows.len() == 0 {
                throw "Source account not found";
            }
            let dest_rows = execute(`SELECT id FROM accounts WHERE id = ${to_id}`);
            if dest_rows.len() == 0 {
                throw "Destination account not found";
            }
            let balance = src_rows[0].balance;
            if balance < amount {
                throw `Insufficient funds: Balance is ${balance}, required ${amount}`;
            }
            execute(`UPDATE accounts SET balance = balance - ${amount} WHERE id = ${from_id}`);
            execute(`UPDATE accounts SET balance = balance + ${amount} WHERE id = ${to_id}`);
        '"#,
        (),
    )?;

    // Test CALL in transaction
    let tx = db.begin()?;
    tx.lock()
        .unwrap()
        .execute("CALL transfer(1, 2, 300.0)", ())?;
    tx.lock().unwrap().commit()?;

    // Verify transfer happened
    let alice_balance: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 1", ())?;
    assert_eq!(alice_balance, 700.0);
    let bob_balance: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 2", ())?;
    assert_eq!(bob_balance, 800.0);

    Ok(())
}

#[test]
fn test_procedure_transaction_rollback() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("memory://")?;

    // Create accounts table
    db.execute(
        "CREATE TABLE accounts (id INTEGER PRIMARY KEY, name TEXT, balance FLOAT)",
        (),
    )?;
    db.execute(
        "INSERT INTO accounts VALUES (1, 'Alice', 1000.0), (2, 'Bob', 500.0)",
        (),
    )?;

    // Create transfer procedure
    db.execute(
        r#"CREATE PROCEDURE transfer(from_id INTEGER, to_id INTEGER, amount FLOAT)
        LANGUAGE rhai AS '
            let src_rows = execute(`SELECT balance FROM accounts WHERE id = ${from_id}`);
            if src_rows.len() == 0 {
                throw "Source account not found";
            }
            let dest_rows = execute(`SELECT id FROM accounts WHERE id = ${to_id}`);
            if dest_rows.len() == 0 {
                throw "Destination account not found";
            }
            let balance = src_rows[0].balance;
            if balance < amount {
                throw `Insufficient funds: Balance is ${balance}, required ${amount}`;
            }
            execute(`UPDATE accounts SET balance = balance - ${amount} WHERE id = ${from_id}`);
            execute(`UPDATE accounts SET balance = balance + ${amount} WHERE id = ${to_id}`);
        '"#,
        (),
    )?;

    // Verify initial balances
    let alice_initial: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 1", ())?;
    let bob_initial: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 2", ())?;
    assert_eq!(alice_initial, 1000.0);
    assert_eq!(bob_initial, 500.0);

    // Begin transaction and call procedure
    let tx = db.begin()?;
    tx.lock()
        .unwrap()
        .execute("CALL transfer(1, 2, 200.0)", ())?;

    // Verify changes are visible inside transaction
    let alice_in_tx: f64 = tx
        .lock()
        .unwrap()
        .query_one("SELECT balance FROM accounts WHERE id = 1", ())?;
    let bob_in_tx: f64 = tx
        .lock()
        .unwrap()
        .query_one("SELECT balance FROM accounts WHERE id = 2", ())?;
    assert_eq!(alice_in_tx, 800.0);
    assert_eq!(bob_in_tx, 700.0);

    // Rollback transaction
    tx.lock().unwrap().rollback()?;

    // Verify changes do not persist after rollback
    let alice_after_rollback: f64 =
        db.query_one("SELECT balance FROM accounts WHERE id = 1", ())?;
    let bob_after_rollback: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 2", ())?;

    assert_eq!(
        alice_after_rollback, 1000.0,
        "Changes should not persist after rollback"
    );
    assert_eq!(
        bob_after_rollback, 500.0,
        "Changes should not persist after rollback"
    );

    Ok(())
}

#[test]
fn test_show_procedures_and_routines() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("memory://")?;

    // Create a procedure
    db.execute("CREATE PROCEDURE test_proc(param1 INTEGER, param2 TEXT) LANGUAGE rhai AS '// Simple procedure'", ())?;

    // Check if procedure was created
    let check = db.query(
        "SELECT name FROM _sys_procedures WHERE name = 'TEST_PROC'",
        (),
    )?;
    let mut count = 0;
    for _ in check {
        count += 1;
    }
    assert_eq!(count, 1, "Procedure should be created");

    // Test SHOW PROCEDURES
    let result = db.query("SHOW PROCEDURES", ())?;
    let mut found = false;
    for row_result in result {
        let row = row_result?;
        if let Ok(Value::Text(name)) = row.get::<Value>(0) {
            if name.as_ref() == "TEST_PROC" {
                found = true;
                // Check args format
                if let Ok(Value::Text(args)) = row.get::<Value>(1) {
                    assert!(args.as_ref().contains("param1 INTEGER"));
                    assert!(args.as_ref().contains("param2 TEXT"));
                }
                // Check language
                if let Ok(Value::Text(lang)) = row.get::<Value>(2) {
                    assert_eq!(lang.as_ref(), "rhai");
                }
                // Check body
                if let Ok(Value::Text(body)) = row.get::<Value>(3) {
                    assert!(body.as_ref().contains("Simple procedure"));
                }
                // Check schema
                if let Ok(Value::Text(schema)) = row.get::<Value>(4) {
                    assert_eq!(schema.as_ref(), "public");
                }
            }
        }
    }
    assert!(found, "test_proc should be found in SHOW PROCEDURES");

    // Test information_schema.routines
    let result = db.query(
        "SELECT routine_name FROM information_schema.routines WHERE routine_name = 'TEST_PROC'",
        (),
    )?;
    let mut found_routine = false;
    for row_result in result {
        let row = row_result?;
        if let Ok(Value::Text(name)) = row.get::<Value>(0) {
            if name.as_ref() == "TEST_PROC" {
                found_routine = true;
            }
        }
    }
    assert!(
        found_routine,
        "TEST_PROC should be found in information_schema.routines"
    );

    Ok(())
}

#[test]
fn test_routine_aliases() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("memory://")?;

    // Test CREATE ROUTINE as alias for CREATE PROCEDURE
    db.execute(
        "CREATE ROUTINE test_routine_alias(param1 INTEGER) LANGUAGE rhai AS '// test'",
        (),
    )?;

    // Test SHOW ROUTINES as alias for SHOW PROCEDURES
    let result = db.query("SHOW ROUTINES", ())?;
    let mut found = false;
    for row_result in result {
        let row = row_result?;
        if let Ok(Value::Text(name)) = row.get::<Value>(0) {
            if name.as_ref() == "TEST_ROUTINE_ALIAS" {
                found = true;
                // Check that it's recognized as a procedure
                if let Ok(Value::Text(lang)) = row.get::<Value>(2) {
                    assert_eq!(lang.as_ref(), "rhai");
                }
            }
        }
    }
    assert!(
        found,
        "ROUTINE should be created and shown via SHOW ROUTINES"
    );

    // Test DROP ROUTINE as alias for DROP PROCEDURE
    db.execute("DROP ROUTINE test_routine_alias", ())?;

    // Verify it's gone
    let result = db.query("SHOW PROCEDURES", ())?;
    let mut found_after_drop = false;
    for row_result in result {
        let row = row_result?;
        if let Ok(Value::Text(name)) = row.get::<Value>(0) {
            if name.as_ref() == "TEST_ROUTINE_ALIAS" {
                found_after_drop = true;
            }
        }
    }
    assert!(
        !found_after_drop,
        "ROUTINE should be dropped via DROP ROUTINE"
    );

    Ok(())
}
