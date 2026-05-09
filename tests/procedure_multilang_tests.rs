use oxibase::api::Database;
use oxibase::core::Value;

#[test]
#[cfg(feature = "js")]
fn test_js_procedure() {
    let mut db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE PROCEDURE multiply_js(a INT, b INT, OUT res INT) 
        LANGUAGE js 
        AS '
            res = a * b;
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create js procedure: {:?}", res.err());

    let call_sql = "CALL multiply_js(5, 4, 0);";
    let res = db.query(call_sql, ());
    assert!(res.is_ok(), "Failed to call js procedure: {:?}", res.err());
    
    let mut results = res.unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_int64().unwrap(), 20);
}

#[test]
#[cfg(feature = "python")]
fn test_python_procedure() {
    let mut db = Database::open_in_memory().unwrap();

    let create_sql = r#"
        CREATE PROCEDURE concat_py(a TEXT, b TEXT, OUT res TEXT) 
        LANGUAGE python 
        AS '
res = a + " " + b
        ';
    "#;

    let res = db.execute(create_sql, ());
    assert!(res.is_ok(), "Failed to create python procedure: {:?}", res.err());

    let call_sql = "CALL concat_py('hello', 'world', '');";
    let res = db.query(call_sql, ());
    assert!(res.is_ok(), "Failed to call python procedure: {:?}", res.err());
    
    let mut results = res.unwrap();
    let row = results.next().unwrap().unwrap();
    assert_eq!(row.get::<Value>(0).unwrap().as_str().unwrap(), "hello world");
}
