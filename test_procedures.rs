use oxibase::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open_in_memory()?;
    
    // Test CREATE PROCEDURE parsing
    let result = db.execute("CREATE PROCEDURE test_proc(a INTEGER, b TEXT) LANGUAGE RHAI AS 'execute(\"INSERT INTO test VALUES (\" + a + \", '\" + b + \"')\");'", ());
    println!("CREATE PROCEDURE result: {:?}", result);
    
    // Test CALL parsing  
    let result = db.execute("CALL test_proc(42, 'hello')", ());
    println!("CALL PROCEDURE result: {:?}", result);
    
    Ok(())
}
