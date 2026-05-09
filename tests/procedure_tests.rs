use oxibase::api::Database;

#[test]
fn test_create_and_call_procedure() {
    let db = Database::open_in_memory().unwrap();

    // Create a basic stored procedure using Rhai
    let create_sql = r#"
        CREATE PROCEDURE my_proc() 
        LANGUAGE rhai 
        AS '
            let a = 10; 
            let b = 20; 
            let c = a + b; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create procedure: {:?}", res.err());

    // Call the stored procedure
    let call_sql = "CALL my_proc();";
    let res = db.execute(call_sql, ());
    assert!(res.is_ok(), "Failed to call procedure: {:?}", res.err());
}

#[test]
fn test_procedure_with_arguments() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE PROCEDURE add_numbers(a INT, b INT, OUT res INT) 
        LANGUAGE rhai 
        AS '
            res = a + b;
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create procedure: {:?}", res.err());

    let call_sql = "CALL add_numbers(10, 5, 0);";
    let res = db.query(call_sql, ());
    assert!(res.is_ok(), "Failed to call procedure: {:?}", res.err());

    let mut results = res.unwrap();
    assert_eq!(results.columns(), &["res"]);

    // Read the first row
    let row = results.next().unwrap().unwrap();
    assert_eq!(
        row.get::<oxibase::core::Value>(0)
            .unwrap()
            .as_int64()
            .unwrap(),
        15
    );
}

#[test]
fn test_rhai_sql_execution() {
    let db = oxibase::api::Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE rhai_logs(id INTEGER PRIMARY KEY AUTO_INCREMENT, msg TEXT);", ()).unwrap();

    let create_sql = r#"
        CREATE PROCEDURE log_rhai(msg TEXT) 
        LANGUAGE rhai 
        AS '
            oxibase.execute("INSERT INTO rhai_logs(msg) VALUES (\"Hello Rhai\")");
        ';
    "#;

    db.execute(create_sql, ()).unwrap();
    db.execute("CALL log_rhai('Hello Rhai');", ()).unwrap();

    let mut results = db.query("SELECT msg FROM rhai_logs;", ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<oxibase::core::Value>(0).unwrap().as_str().unwrap(), "Hello Rhai");
}
