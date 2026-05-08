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
