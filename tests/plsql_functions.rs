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

#[test]
fn test_plsql_function_with_different_types() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE FUNCTION test_types(f FLOAT, t TIMESTAMP, j JSON) RETURNS FLOAT
        LANGUAGE plsql 
        AS ' 
        DECLARE
            f_val FLOAT := f;
            t_val TIMESTAMP := t;
            j_val JSON := j;
        BEGIN 
            IF f_val > 1.5 THEN
                f_val := f_val + 2.5;
            END IF;
            RETURN f_val;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create function: {:?}", res.err());

    // Call with 2.0 (should become 4.5), a timestamp, and a json
    let call_sql = "SELECT test_types(2.0, '2023-01-01T00:00:00Z', '{\"a\": 1}');";
    let res = db.query(call_sql, ());
    assert!(res.is_ok(), "Failed to call function: {:?}", res.err());

    let mut results = res.unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_float64().unwrap(), 4.5);

    // Call with 1.0 (should remain 1.0)
    let call_sql = "SELECT test_types(1.0, '2023-01-01T00:00:00Z', '{\"a\": 1}');";
    let mut results = db.query(call_sql, ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_float64().unwrap(), 1.0);
}

#[test]
fn test_plsql_arithmetic_all_types() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE FUNCTION math_ops(i1 INT, i2 INT, f1 FLOAT, f2 FLOAT) RETURNS FLOAT
        LANGUAGE plsql 
        AS ' 
        DECLARE
            res FLOAT := 0.0;
        BEGIN 
            res := i1 + i2;          -- int + int
            res := res + f1;         -- int + float (res is coerced to float, so float + float)
            res := res - i1;         -- float - int
            res := res * f2;         -- float * float
            res := res / 2;          -- float / int
            RETURN res;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create function: {:?}", res.err());

    // i1=10, i2=5, f1=2.5, f2=2.0
    // step 1: 10 + 5 = 15.0
    // step 2: 15.0 + 2.5 = 17.5
    // step 3: 17.5 - 10 = 7.5
    // step 4: 7.5 * 2.0 = 15.0
    // step 5: 15.0 / 2 = 7.5
    let call_sql = "SELECT math_ops(10, 5, 2.5, 2.0);";
    let mut results = db.query(call_sql, ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_float64().unwrap(), 7.5);
}

#[test]
fn test_plsql_comparisons() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE FUNCTION test_comps(i INT, f FLOAT) RETURNS BOOLEAN
        LANGUAGE plsql 
        AS ' 
        DECLARE
            res BOOLEAN := false;
        BEGIN 
            IF i = f THEN
                res := true;
            END IF;
            IF i < f THEN
                res := false;
            END IF;
            IF f > i THEN
                res := false;
            END IF;
            RETURN res;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create function: {:?}", res.err());

    let call_sql = "SELECT test_comps(5, 5.0);";
    let mut results = db.query(call_sql, ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert!(row.get::<Value>(0).unwrap().as_boolean().unwrap());
}
