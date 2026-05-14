use oxibase::Database;

#[test]
fn test_sequence_lifecycle() {
    let db = Database::open("memory://").unwrap();

    // 1. Create sequence
    db.execute(
        "CREATE SEQUENCE my_seq START WITH 10 INCREMENT BY 5 MAXVALUE 25",
        (),
    )
    .unwrap();

    // 2. NextVal should increment
    let val1: i64 = db.query_one("SELECT NEXTVAL('my_seq')", ()).unwrap();
    assert_eq!(val1, 10);

    let val2: i64 = db.query_one("SELECT NEXTVAL('my_seq')", ()).unwrap();
    assert_eq!(val2, 15);

    let val3: i64 = db.query_one("SELECT NEXTVAL('my_seq')", ()).unwrap();
    assert_eq!(val3, 20);

    let val4: i64 = db.query_one("SELECT NEXTVAL('my_seq')", ()).unwrap();
    assert_eq!(val4, 25);

    // Should fail without CYCLE
    let err = db.query_one::<i64, _>("SELECT NEXTVAL('my_seq')", ());
    assert!(
        err.is_err(),
        "Expected error when exceeding maxvalue without CYCLE"
    );

    // 3. CurrVal should return 25
    let curr: i64 = db.query_one("SELECT CURRVAL('my_seq')", ()).unwrap();
    assert_eq!(curr, 25);

    // 4. SetVal should reset
    let set: i64 = db.query_one("SELECT SETVAL('my_seq', 12)", ()).unwrap();
    assert_eq!(set, 12);

    let next_after_set: i64 = db.query_one("SELECT NEXTVAL('my_seq')", ()).unwrap();
    // Default is_called=true, so nextval increments first!
    assert_eq!(next_after_set, 17);

    // 5. Check Information Schema
    let mut rows = db.query("SELECT sequence_name, current_value FROM information_schema.sequences WHERE sequence_name = 'my_seq'", ()).unwrap();
    let row = rows.next().unwrap().unwrap();
    let name: String = row.get(0).unwrap();
    let current_val: i64 = row.get(1).unwrap();
    assert_eq!(name, "my_seq");
    assert_eq!(current_val, 17);

    // 6. Alter sequence
    db.execute(
        "ALTER SEQUENCE my_seq RESTART WITH 100 INCREMENT BY 1 CYCLE",
        (),
    )
    .unwrap();
    let val5: i64 = db.query_one("SELECT NEXTVAL('my_seq')", ()).unwrap();
    assert_eq!(val5, 100);

    // 7. Drop sequence
    db.execute("DROP SEQUENCE my_seq", ()).unwrap();

    // Query Information Schema to confirm deletion
    let mut rows2 = db
        .query(
            "SELECT sequence_name FROM information_schema.sequences WHERE sequence_name = 'my_seq'",
            (),
        )
        .unwrap();
    assert!(rows2.next().is_none());
}
