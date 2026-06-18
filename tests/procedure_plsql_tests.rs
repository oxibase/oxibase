// Copyright 2025 Oxibase Contributors
use oxibase::api::Database;
use oxibase::core::Value;

#[test]
fn test_plsql_procedure() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE PROCEDURE check_val(val INT, OUT is_positive BOOLEAN) 
        LANGUAGE plsql 
        AS ' 
        BEGIN 
            IF val > 0 THEN 
                is_positive := true; 
            ELSE 
                is_positive := false; 
            END IF; 
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create procedure: {:?}", res.err());

    let call_sql_true = "CALL check_val(5, false);";
    let res_true = db.query(call_sql_true, ());
    assert!(
        res_true.is_ok(),
        "Failed to call procedure: {:?}",
        res_true.err()
    );

    let mut results = res_true.unwrap();
    assert_eq!(results.columns(), &["is_positive"]);
    let row = results.next().unwrap().unwrap();
    assert!(row.get::<Value>(0).unwrap().as_boolean().unwrap());

    let call_sql_false = "CALL check_val(-5, true);";
    let res_false = db.query(call_sql_false, ());
    assert!(
        res_false.is_ok(),
        "Failed to call procedure: {:?}",
        res_false.err()
    );

    let mut results = res_false.unwrap();
    let row = results.next().unwrap().unwrap();
    assert!(!row.get::<Value>(0).unwrap().as_boolean().unwrap());
}

#[test]
fn test_plsql_sql_execution() {
    let db = Database::open_in_memory().unwrap();

    db.execute(
        "CREATE TABLE logs(id INTEGER PRIMARY KEY AUTO_INCREMENT, message TEXT);",
        (),
    )
    .unwrap();

    let create_sql = r#"
        CREATE PROCEDURE log_event(msg TEXT) 
        LANGUAGE plsql 
        AS ' 
        BEGIN 
            INSERT INTO logs(message) VALUES (msg); 
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create procedure: {:?}", res.err());

    let call_sql = "CALL log_event('Hello from PL/SQL!');";
    let res = db.execute(call_sql, ());
    assert!(res.is_ok(), "Failed to call procedure: {:?}", res.err());

    let mut results = db.query("SELECT message FROM logs;", ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(
        row.get::<Value>(0).unwrap().as_str().unwrap(),
        "Hello from PL/SQL!"
    );
}

#[test]
fn test_plsql_declare_and_while() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE PROCEDURE factorial(n INT, OUT res INT) 
        LANGUAGE plsql 
        AS ' 
        DECLARE
            counter INT := n;
            acc INT := 1;
        BEGIN 
            WHILE counter > 0 LOOP
                acc := acc * counter;
                counter := counter - 1;
            END LOOP;
            res := acc;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create procedure: {:?}", res.err());

    let call_sql = "CALL factorial(5, 0);";
    let res = db.query(call_sql, ());
    assert!(res.is_ok(), "Failed to call procedure: {:?}", res.err());

    let mut results = res.unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_int64().unwrap(), 120);
}

#[test]
fn test_plsql_sql_substitution() {
    let db = Database::open_in_memory().unwrap();

    db.execute(
        "CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT, active BOOLEAN);",
        (),
    )
    .unwrap();

    db.execute(
        "INSERT INTO users (id, name, active) VALUES (1, 'Alice', true), (2, 'Bob', false), (3, 'Charlie', true);",
        (),
    ).unwrap();

    let create_sql = r#"
        CREATE PROCEDURE delete_inactive(OUT deleted_count INT) 
        LANGUAGE plsql 
        AS ' 
        DECLARE
            target_status BOOLEAN := false;
        BEGIN 
            -- The parser translates this into a standard SQL statement where target_status is substituted
            DELETE FROM users WHERE active = target_status;
            -- Setting out param to arbitrary value for test purposes since we do not have ROW_COUNT yet
            deleted_count := 1;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create procedure: {:?}", res.err());

    let call_sql = "CALL delete_inactive(0);";
    let res = db.query(call_sql, ());
    assert!(res.is_ok(), "Failed to call procedure: {:?}", res.err());

    let mut results = db.query("SELECT count(*) FROM users;", ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_int64().unwrap(), 2);
}

#[test]
fn test_plsql_transaction_commit_rollback() {
    let db = Database::open_in_memory().unwrap();

    db.execute(
        "CREATE TABLE tx_test_plsql(id INTEGER PRIMARY KEY, val TEXT);",
        (),
    )
    .unwrap();

    let create_sql = r#"
        CREATE PROCEDURE tx_proc_plsql() 
        LANGUAGE plsql 
        AS ' 
        BEGIN 
            INSERT INTO tx_test_plsql(id, val) VALUES (1, ''first'');
            COMMIT;
            
            BEGIN; -- Should be a no-op
            INSERT INTO tx_test_plsql(id, val) VALUES (2, ''second'');
            ROLLBACK;
            
            INSERT INTO tx_test_plsql(id, val) VALUES (3, ''third'');
            COMMIT;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create procedure: {:?}", res.err());

    let call_sql = "CALL tx_proc_plsql();";
    let res = db.execute(call_sql, ());
    assert!(res.is_ok(), "Failed to call procedure: {:?}", res.err());

    let mut results = db
        .query("SELECT id, val FROM tx_test_plsql ORDER BY id;", ())
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
fn test_plsql_transaction_explicit_nested_error() {
    let db = Database::open_in_memory().unwrap();

    db.execute(
        "CREATE TABLE tx_test_nested(id INTEGER PRIMARY KEY, val TEXT);",
        (),
    )
    .unwrap();

    let create_sql = r#"
        CREATE PROCEDURE tx_proc_nested() 
        LANGUAGE plsql 
        AS ' 
        BEGIN 
            INSERT INTO tx_test_nested(id, val) VALUES (1, ''first'');
            COMMIT;
        END; 
        ';
    "#;

    db.execute(create_sql, ()).unwrap();

    // Call inside explicit transaction should fail!
    let _ = db.execute("BEGIN;", ()).unwrap();

    let res = db.execute("CALL tx_proc_nested();", ());
    assert!(
        res.is_err(),
        "Expected error when calling procedure with transaction control from explicit transaction"
    );

    let err_msg = res.err().unwrap().to_string();
    assert!(
        err_msg.contains("invalid transaction termination"),
        "Unexpected error: {}",
        err_msg
    );

    let _ = db.execute("ROLLBACK;", ()).unwrap();
}

#[test]
fn test_plsql_procedure_print_stdout() {
    let db = Database::open_in_memory().unwrap();
    oxibase::functions::context::clear_stdout();

    let create_sql = r#"
        CREATE PROCEDURE test_stdout() 
        LANGUAGE plsql 
        AS ' 
        DECLARE
            counter INT := 0;
        BEGIN 
            PRINT ''Starting plsql trace'';
            counter := counter + 5;
            RAISE NOTICE counter;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create procedure: {:?}", res.err());

    let call_sql = "CALL test_stdout();";
    let res = db.execute(call_sql, ());
    assert!(res.is_ok(), "Failed to call procedure: {:?}", res.err());

    let stdout = oxibase::functions::context::get_stdout();
    assert!(stdout.contains("Starting plsql trace"));
    assert!(stdout.contains("5"));
}
