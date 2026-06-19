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

#[test]
fn test_plsql_uninitialized_declarations() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE FUNCTION test_defaults() RETURNS FLOAT
        LANGUAGE plsql 
        AS ' 
        DECLARE
            f FLOAT;
            d DECIMAL;
            n NUMERIC;
            j JSON;
            t TIMESTAMP;
            res FLOAT;
        BEGIN 
            -- Just setting res to something to return
            res := 1.0;
            
            IF f = NULL THEN
                res := res + 1.0;
            END IF;
            IF d = NULL THEN
                res := res + 1.0;
            END IF;
            IF n = NULL THEN
                res := res + 1.0;
            END IF;
            IF j = NULL THEN
                res := res + 1.0;
            END IF;
            IF t = NULL THEN
                res := res + 1.0;
            END IF;
            
            RETURN res;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create function: {:?}", res.err());

    let call_sql = "SELECT test_defaults();";
    let mut results = db.query(call_sql, ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_float64().unwrap(), 6.0);
}

#[test]
fn test_plsql_extended_comparisons() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE FUNCTION test_ext_comps(i INT, f FLOAT) RETURNS BOOLEAN
        LANGUAGE plsql 
        AS ' 
        DECLARE
            res BOOLEAN := false;
        BEGIN 
            IF i <= f THEN
                res := true;
            END IF;
            IF f >= i THEN
                res := true;
            END IF;
            IF i != 100.0 THEN
                res := true;
            END IF;
            IF i <> 100.0 THEN
                res := true;
            END IF;
            RETURN res;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create function: {:?}", res.err());

    let call_sql = "SELECT test_ext_comps(5, 5.0);";
    let mut results = db.query(call_sql, ()).unwrap();
    let row = results.next().unwrap().unwrap();
    assert!(row.get::<Value>(0).unwrap().as_boolean().unwrap());
}

#[test]
fn test_plsql_arithmetic_errors() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE FUNCTION test_div_zero() RETURNS INT
        LANGUAGE plsql 
        AS ' 
        BEGIN 
            RETURN 1 / 0;
        END; 
        ';
    "#;
    db.execute(create_sql, ()).unwrap();
    let res = db.query("SELECT test_div_zero();", ()).unwrap();
    let mut rows = res;
    let row = rows.next().unwrap().unwrap();
    assert_eq!(
        row.get::<Value>(0).unwrap(),
        Value::Null(oxibase::core::DataType::Null)
    );

    let create_sql2 = r#"
        CREATE FUNCTION test_invalid_add() RETURNS INT
        LANGUAGE plsql 
        AS ' 
        DECLARE
            b BOOLEAN := true;
        BEGIN 
            RETURN 1 + b;
        END; 
        ';
    "#;
    db.execute(create_sql2, ()).unwrap();
    let mut res2 = db.query("SELECT test_invalid_add();", ()).unwrap();
    let row2 = res2.next().unwrap().unwrap();
    assert_eq!(
        row2.get::<Value>(0).unwrap(),
        Value::Null(oxibase::core::DataType::Null)
    );

    let create_sql3 = r#"
        CREATE FUNCTION test_invalid_sub() RETURNS INT
        LANGUAGE plsql 
        AS ' 
        DECLARE
            b BOOLEAN := true;
        BEGIN 
            RETURN 1 - b;
        END; 
        ';
    "#;
    db.execute(create_sql3, ()).unwrap();
    let mut res3 = db.query("SELECT test_invalid_sub();", ()).unwrap();
    let row3 = res3.next().unwrap().unwrap();
    assert_eq!(
        row3.get::<Value>(0).unwrap(),
        Value::Null(oxibase::core::DataType::Null)
    );

    let create_sql4 = r#"
        CREATE FUNCTION test_invalid_mul() RETURNS INT
        LANGUAGE plsql 
        AS ' 
        DECLARE
            b BOOLEAN := true;
        BEGIN 
            RETURN 1 * b;
        END; 
        ';
    "#;
    db.execute(create_sql4, ()).unwrap();
    let mut res4 = db.query("SELECT test_invalid_mul();", ()).unwrap();
    let row4 = res4.next().unwrap().unwrap();
    assert_eq!(
        row4.get::<Value>(0).unwrap(),
        Value::Null(oxibase::core::DataType::Null)
    );

    let create_sql5 = r#"
        CREATE FUNCTION test_invalid_div() RETURNS INT
        LANGUAGE plsql 
        AS ' 
        DECLARE
            b BOOLEAN := true;
        BEGIN 
            RETURN 1 / b;
        END; 
        ';
    "#;
    db.execute(create_sql5, ()).unwrap();
    let mut res5 = db.query("SELECT test_invalid_div();", ()).unwrap();
    let row5 = res5.next().unwrap().unwrap();
    assert_eq!(
        row5.get::<Value>(0).unwrap(),
        Value::Null(oxibase::core::DataType::Null)
    );
}

#[test]
fn test_plsql_arithmetic_all_combinations() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE FUNCTION math_ops_all(i INT, f FLOAT) RETURNS FLOAT
        LANGUAGE plsql 
        AS ' 
        DECLARE
            res FLOAT := 0.0;
            tmp INT := 0;
        BEGIN 
            -- Int + Float, Float + Int
            res := i + f;
            res := f + i;
            
            -- Float - Float, Int - Float, Float - Int
            res := f - f;
            res := i - f;
            res := f - i;

            -- Int * Int, Int * Float, Float * Int, Float * Float
            tmp := i * i;
            res := i * f;
            res := f * i;
            res := f * f;

            -- Int / Int, Int / Float, Float / Int, Float / Float
            tmp := i / i;
            res := i / f;
            res := f / i;
            res := f / f;

            RETURN res;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create function: {:?}", res.err());

    let call_sql = "SELECT math_ops_all(10, 2.0);";
    let mut results = db.query(call_sql, ()).unwrap();
    let row = results.next().unwrap().unwrap();
    // f / f -> 2.0 / 2.0 = 1.0
    assert_eq!(row.get::<Value>(0).unwrap().as_float64().unwrap(), 1.0);
}

#[test]
fn test_plsql_random() {
    let db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE FUNCTION test_random_plsql() RETURNS FLOAT
        LANGUAGE plsql 
        AS ' 
        DECLARE
            r FLOAT;
        BEGIN 
            r := random();
            RETURN r;
        END; 
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create function: {:?}", res.err());

    let call_sql = "SELECT test_random_plsql() AS r;";
    let mut results = db.query(call_sql, ()).unwrap();
    let row = results.next().unwrap().unwrap();
    let val = row.get::<Value>(0).unwrap().as_float64().unwrap();
    assert!(
        (0.0..1.0).contains(&val),
        "Expected float in [0, 1), got {}",
        val
    );
}
