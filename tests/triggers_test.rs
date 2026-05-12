use oxibase::core::{DataType, Value};
use oxibase::executor::Executor;
use oxibase::storage::mvcc::engine::MVCCEngine;
use std::sync::Arc;

fn setup_executor() -> Executor {
    let engine = MVCCEngine::in_memory();
    engine.open_engine().unwrap();
    Executor::new(Arc::new(engine))
}

#[test]
fn test_create_and_drop_trigger() {
    let executor = setup_executor();
    executor.execute("CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
    
    // Create trigger
    let result = executor.execute(r#"
        CREATE TRIGGER test_trigger
        BEFORE INSERT ON test_table
        FOR EACH ROW
        LANGUAGE rhai
        AS '
            if NEW.id < 0 {
                throw "Negative ID not allowed";
            }
        ';
    "#);
    
    assert!(result.is_ok(), "Failed to create trigger: {:?}", result.err());
    
    // Test validation
    let insert_err = executor.execute("INSERT INTO test_table (id, name) VALUES (-1, 'test')");
    assert!(insert_err.is_err());
    if let Err(e) = insert_err {
        assert!(e.to_string().contains("Negative ID not allowed"));
    }
    
    // Test valid insert
    let insert_ok = executor.execute("INSERT INTO test_table (id, name) VALUES (1, 'test')");
    assert!(insert_ok.is_ok());
    
    // Drop trigger
    let drop_result = executor.execute("DROP TRIGGER test_trigger ON test_table");
    assert!(drop_result.is_ok());
}

#[test]
fn test_data_transformation_trigger() {
    let executor = setup_executor();
    executor.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
    
    executor.execute(r#"
        CREATE TRIGGER normalize_name
        BEFORE INSERT ON users
        FOR EACH ROW
        LANGUAGE rhai
        AS '
            NEW.name = "PREFIX_" + NEW.name;
        ';
    "#).unwrap();
    
    executor.execute("INSERT INTO users (id, name) VALUES (1, 'alice')").unwrap();
    
    let mut result = executor.execute("SELECT name FROM users WHERE id = 1").unwrap();
    assert!(result.next());
    assert_eq!(result.row().get(0), Some(&Value::text("PREFIX_alice")));
}

#[test]
fn test_audit_trigger() {
    let executor = setup_executor();
    executor.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, price FLOAT)").unwrap();
    executor.execute("CREATE TABLE audit_log (product_id INTEGER, old_price FLOAT, new_price FLOAT)").unwrap();
    
    executor.execute(r#"
        CREATE TRIGGER audit_price
        AFTER UPDATE ON products
        FOR EACH ROW
        LANGUAGE rhai
        AS '
            if OLD.price != NEW.price {
                oxibase::execute("INSERT INTO audit_log (product_id, old_price, new_price) VALUES (" + OLD.id + ", " + OLD.price + ", " + NEW.price + ")");
            }
        ';
    "#).unwrap();
    
    executor.execute("INSERT INTO products (id, price) VALUES (1, 10.0)").unwrap();
    
    // Update price
    executor.execute("UPDATE products SET price = 15.0 WHERE id = 1").unwrap();
    
    // Check audit log
    let mut result = executor.execute("SELECT old_price, new_price FROM audit_log WHERE product_id = 1").unwrap();
    assert!(result.next());
    let row = result.row();
    assert_eq!(row.get(0), Some(&Value::Float(10.0)));
    assert_eq!(row.get(1), Some(&Value::Float(15.0)));
}
