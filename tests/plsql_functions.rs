// Copyright 2025 Oxibase Contributors
use oxibase::api::Database;
use oxibase::core::Value;

#[test]
fn test_plsql_scalar_function_basic() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE FUNCTION add_numbers(a INT, b INT) RETURNS INT
        LANGUAGE plsql 
        AS ' 
        BEGIN 
            RETURN a + b; 
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create function: {:?}", res.err());

    let call_sql = "SELECT add_numbers(5, 10) AS result;";
    let res = db.query(call_sql, ());
    assert!(res.is_ok(), "Failed to call function: {:?}", res.err());

    let mut results = res.unwrap();
    assert_eq!(results.columns(), &["result"]);
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_int64().unwrap(), 15);
}

#[test]
fn test_plsql_scalar_function_control_flow() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE FUNCTION factorial(n INT) RETURNS INT
        LANGUAGE plsql 
        AS ' 
        DECLARE
            counter INT := n;
            acc INT := 1;
        BEGIN 
            IF n <= 0 THEN
                RETURN 1;
            END IF;

            WHILE counter > 0 LOOP
                acc := acc * counter;
                counter := counter - 1;
            END LOOP;
            RETURN acc;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create function: {:?}", res.err());

    let call_sql = "SELECT factorial(5) AS f5, factorial(0) AS f0;";
    let mut results = db.query(call_sql, ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_int64().unwrap(), 120);
    assert_eq!(row.get::<Value>(1).unwrap().as_int64().unwrap(), 1);
}
