use oxibase::api::database::Database;

#[test]
fn test_public_schema_fallback() {
    let db = Database::open_in_memory().unwrap();

    // Test 1: Procedure
    db.execute(
        "CREATE PROCEDURE my_proc() LANGUAGE sql AS $$ SELECT 1; $$",
        (),
    )
    .unwrap();
    let result = db
        .query(
            "SELECT schema FROM system.procedures WHERE name = 'MY_PROC'",
            (),
        )
        .unwrap();
    let rows = result.collect_vec().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String>(0).unwrap(), "PUBLIC");

    // Test 2: Function
    db.execute(
        "CREATE FUNCTION my_func() RETURNS INT LANGUAGE sql AS 'SELECT 1;'",
        (),
    )
    .unwrap();
    let result = db
        .query(
            "SELECT schema FROM system.functions WHERE name = 'MY_FUNC'",
            (),
        )
        .unwrap();
    let rows = result.collect_vec().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String>(0).unwrap(), "PUBLIC");

    // Test 3: Table & Trigger
    db.execute("CREATE TABLE users (id INT)", ()).unwrap();
    db.execute(
        "CREATE TRIGGER my_trig BEFORE INSERT ON users FOR EACH ROW LANGUAGE sql AS 'SELECT 1;'",
        (),
    )
    .unwrap();
    let result = db
        .query(
            "SELECT schema FROM system.triggers WHERE name = 'MY_TRIG'",
            (),
        )
        .unwrap();
    let rows = result.collect_vec().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String>(0).unwrap(), "PUBLIC");

    // Test 4: Sequence
    db.execute("CREATE SEQUENCE my_seq", ()).unwrap();
    let result = db.query("SELECT NEXTVAL('my_seq')", ()).unwrap();
    let rows = result.collect_vec().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<i64>(0).unwrap(), 1);

    // Test 5: View
    db.execute("CREATE VIEW my_view AS SELECT 1 AS id", ())
        .unwrap();
    let result = db.query("SELECT id FROM my_view", ()).unwrap();
    let rows = result.collect_vec().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<i64>(0).unwrap(), 1);
}
