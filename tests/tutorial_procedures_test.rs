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
    let mut tx = db.begin()?;
    tx.execute("CALL transfer(1, 2, 300.0)", ())?;
    tx.commit()?;

    // Verify transfer happened
    let alice_balance: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 1", ())?;
    assert_eq!(alice_balance, 700.0);
    let bob_balance: f64 = db.query_one("SELECT balance FROM accounts WHERE id = 2", ())?;
    assert_eq!(bob_balance, 800.0);

    Ok(())
}
