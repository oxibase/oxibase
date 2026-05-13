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
fn test_tutorial_triggers() {
    let executor = setup_executor();

    // Step 1: Validation in Rhai
    let res = executor.execute(
        "CREATE TABLE accounts (
            id INTEGER PRIMARY KEY,
            owner_name TEXT,
            balance FLOAT
        );",
    );
    assert!(res.is_ok());

    let res = executor.execute(
        r#"
        CREATE TRIGGER ensure_positive_balance
            BEFORE INSERT ON accounts
            FOR EACH ROW
            LANGUAGE rhai
        AS '
            if oxibase.ctx["new"].balance < 0.0 {
                throw "Account balance cannot be negative!";
            }
        ';
        "#,
    );
    if let Err(e) = &res {
        println!("Error creating Rhai trigger: {:?}", e);
    }
    assert!(res.is_ok());

    let res = executor
        .execute("INSERT INTO accounts (id, owner_name, balance) VALUES (1, 'Alice', 100.0);");
    assert!(res.is_ok());

    let res = executor
        .execute("INSERT INTO accounts (id, owner_name, balance) VALUES (2, 'Bob', -50.0);");
    assert!(res.is_err());
    let err_msg = res.err().unwrap().to_string();
    assert!(err_msg.contains("Account balance cannot be negative!"));

    // Step 2: Transformation in JS
    let res = executor.execute(
        r#"
        CREATE TRIGGER normalize_owner_name
            BEFORE UPDATE ON accounts
            FOR EACH ROW
            LANGUAGE js
        AS '
            // Force the owner name to be uppercase before saving
            oxibase.ctx.new.owner_name = oxibase.ctx.new.owner_name.toUpperCase();
        ';
        "#,
    );
    assert!(res.is_ok());

    let res = executor.execute("UPDATE accounts SET owner_name = 'alice lowercase' WHERE id = 1;");
    assert!(res.is_ok());

    let mut res = executor
        .execute("SELECT owner_name FROM accounts WHERE id = 1;")
        .unwrap();
    assert!(res.next());
    assert_eq!(res.row().get(0), Some(&Value::text("ALICE LOWERCASE")));

    // Step 3: Audit Logging in Python
    let res = executor.execute(
        "CREATE TABLE audit_log (
            account_id INTEGER,
            old_balance FLOAT,
            new_balance FLOAT,
            changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );",
    );
    assert!(res.is_ok());

    let res = executor.execute(
        r#"
        CREATE TRIGGER log_balance_changes
            AFTER UPDATE ON accounts
            FOR EACH ROW
            LANGUAGE python
        AS '
import oxibase

# Only log if the balance actually changed
if oxibase.ctx.old["balance"] != oxibase.ctx.new["balance"]:
    # Use oxibase.execute to run DML side-effects
    stmt = "INSERT INTO audit_log (account_id, old_balance, new_balance) VALUES (" + str(oxibase.ctx.old["id"]) + ", " + str(oxibase.ctx.old["balance"]) + ", " + str(oxibase.ctx.new["balance"]) + ")"
    oxibase.execute(stmt)
';
        "#
    );
    if let Err(e) = &res {
        println!("Error creating Python trigger: {:?}", e);
    }
    assert!(res.is_ok());

    // Perform an UPDATE that changes the balance
    let res = executor.execute("UPDATE accounts SET balance = 150.0 WHERE id = 1;");
    if let Err(e) = &res {
        println!("Error executing UPDATE for Python trigger: {:?}", e);
    }
    assert!(res.is_ok());

    // Query audit_log to assert logging happened
    let mut res = executor
        .execute("SELECT account_id, old_balance, new_balance FROM audit_log;")
        .unwrap();
    assert!(res.next());
    assert_eq!(res.row().get(0), Some(&Value::Integer(1)));
    assert_eq!(res.row().get(1), Some(&Value::Float(100.0)));
    assert_eq!(res.row().get(2), Some(&Value::Float(150.0)));

    // Step 4: Dropping
    let res = executor.execute("DROP TRIGGER IF EXISTS log_balance_changes ON accounts;");
    assert!(res.is_ok());
}
