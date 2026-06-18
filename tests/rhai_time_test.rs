use oxibase::api::database::Database;
use oxibase::core::Value;

#[test]
fn test_rhai_timestamp() {
    let db = Database::open("memory://rhai_time_test").unwrap();
    db.execute("CREATE FUNCTION test_time() RETURNS BOOLEAN LANGUAGE rhai AS $$
        let t = timestamp();
        sleep(0.1);
        return t.elapsed > 0.0;
    $$;").unwrap();
    let rows = db.execute("SELECT test_time();").unwrap().into_rows();
    assert_eq!(rows[0].get(0), Some(&Value::Boolean(true)));
}
