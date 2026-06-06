use oxibase::api::Database;
fn main() {
    let db = Database::open("memory://").unwrap();
    db.execute("CREATE TABLE t (id INT)", ()).unwrap();
    let mut tx = db.begin().unwrap();
    tx.execute("INSERT INTO t VALUES (1)", ()).unwrap();
    
    let count_star: i64 = tx.query_one("SELECT COUNT(*) FROM t", ()).unwrap();
    let count_id: i64 = tx.query_one("SELECT COUNT(id) FROM t", ()).unwrap();
    
    println!("COUNT(*) = {}", count_star);
    println!("COUNT(id) = {}", count_id);
}
