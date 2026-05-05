// Copyright 2025 Stoolap Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use oxibase::core::Value;
use oxibase::Database;

#[test]
fn test_create_table_with_foreign_key() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)", ())
        .unwrap();
    db.execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id))", ()).unwrap();
}

#[test]
fn test_alter_table_add_foreign_key() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)", ())
        .unwrap();
    db.execute(
        "CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER)",
        (),
    )
    .unwrap();

    // Add constraint
    db.execute(
        "ALTER TABLE orders ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id)",
        (),
    )
    .unwrap();
}

#[test]
fn test_insert_validation_valid() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY)", ())
        .unwrap();
    db.execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id))", ()).unwrap();

    db.execute("INSERT INTO users (id) VALUES (1)", ()).unwrap();
    // Valid insert
    let result = db.execute("INSERT INTO orders (id, user_id) VALUES (100, 1)", ());
    assert!(result.is_ok());
}

#[test]
fn test_insert_validation_invalid() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY)", ())
        .unwrap();
    db.execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id))", ()).unwrap();

    db.execute("INSERT INTO users (id) VALUES (1)", ()).unwrap();
    // Invalid insert (user 2 does not exist)
    let result = db.execute("INSERT INTO orders (id, user_id) VALUES (100, 2)", ());
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("FOREIGN KEY constraint failed"));
}

#[test]
fn test_insert_validation_null() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY)", ())
        .unwrap();
    db.execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id))", ()).unwrap();

    // Null is allowed
    let result = db.execute("INSERT INTO orders (id, user_id) VALUES (100, NULL)", ());
    assert!(result.is_ok());
}

#[test]
fn test_delete_restrict() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY)", ())
        .unwrap();
    db.execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT)", ()).unwrap();

    db.execute("INSERT INTO users (id) VALUES (1)", ()).unwrap();
    db.execute("INSERT INTO orders (id, user_id) VALUES (100, 1)", ())
        .unwrap();

    // Delete should fail because of RESTRICT
    let result = db.execute("DELETE FROM users WHERE id = 1", ());
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Cannot DELETE row from users: referenced by"));
}

#[test]
fn test_delete_cascade() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY)", ())
        .unwrap();
    db.execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE)", ()).unwrap();

    db.execute("INSERT INTO users (id) VALUES (1)", ()).unwrap();
    db.execute("INSERT INTO orders (id, user_id) VALUES (100, 1)", ())
        .unwrap();

    // Delete should succeed and cascade
    let result = db.execute("DELETE FROM users WHERE id = 1", ());
    assert!(result.is_ok());

    let mut select = db.query("SELECT * FROM orders", ()).unwrap();
    assert!(
        select.next().is_none(),
        "Order should have been deleted by CASCADE"
    );
}

#[test]
fn test_delete_set_null() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY)", ())
        .unwrap();
    db.execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL)", ()).unwrap();

    db.execute("INSERT INTO users (id) VALUES (1)", ()).unwrap();
    db.execute("INSERT INTO orders (id, user_id) VALUES (100, 1)", ())
        .unwrap();

    // Delete should succeed and set user_id to NULL
    let result = db.execute("DELETE FROM users WHERE id = 1", ());
    assert!(result.is_ok());

    let mut select = db
        .query("SELECT user_id FROM orders WHERE id = 100", ())
        .unwrap();
    let row = select.next().unwrap().unwrap();
    assert_eq!(row.get_value(0).unwrap(), &Value::null_unknown());
}

#[test]
fn test_self_referencing() {
    let db = Database::open_in_memory().unwrap();
    db.execute("CREATE TABLE employees (id INTEGER PRIMARY KEY, manager_id INTEGER, FOREIGN KEY (manager_id) REFERENCES employees(id) ON DELETE SET NULL)", ()).unwrap();

    db.execute(
        "INSERT INTO employees (id, manager_id) VALUES (1, NULL)",
        (),
    )
    .unwrap();
    db.execute("INSERT INTO employees (id, manager_id) VALUES (2, 1)", ())
        .unwrap();

    // Delete manager should set manager_id to NULL for employee 2
    db.execute("DELETE FROM employees WHERE id = 1", ())
        .unwrap();

    let mut select = db
        .query("SELECT manager_id FROM employees WHERE id = 2", ())
        .unwrap();
    let row = select.next().unwrap().unwrap();
    assert_eq!(row.get_value(0).unwrap(), &Value::null_unknown());
}

#[test]
fn test_schema_evolution_and_transaction_inserts() {
    let db = Database::open_in_memory().unwrap();

    // 1. Create categories
    db.execute(
        "CREATE TABLE categories (id INTEGER PRIMARY KEY, name TEXT NOT NULL, description TEXT)",
        (),
    )
    .unwrap();
    db.execute(
        "INSERT INTO categories (id, name, description) VALUES (1, 'Electronics', 'Electronic devices and gadgets'), (2, 'Accessories', 'Peripherals and accessories for devices')",
        ()
    ).unwrap();

    // 2. Create products
    db.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT NOT NULL, description TEXT, price FLOAT NOT NULL, category TEXT, in_stock BOOLEAN, created_at TIMESTAMP)", ()).unwrap();
    db.execute(
        "INSERT INTO products (id, name, description, price, category, in_stock, created_at) VALUES 
        (1, 'Laptop', 'High-performance laptop with 16GB RAM', 1299.99, 'Electronics', TRUE, NOW()), 
        (2, 'Smartphone', '5G smartphone with 128GB storage', 799.99, 'Electronics', TRUE, NOW()), 
        (3, 'Headphones', 'Wireless noise-cancelling headphones', 249.99, 'Accessories', TRUE, NOW()), 
        (4, 'Monitor', '27-inch 4K monitor', 349.99, 'Electronics', FALSE, NOW()), 
        (5, 'Keyboard', 'Mechanical gaming keyboard', 129.99, 'Accessories', TRUE, NOW())",
        ()
    ).unwrap();

    // 3. Evolve schema: Add column and populate it
    db.execute("ALTER TABLE products ADD COLUMN category_id INTEGER", ())
        .unwrap();
    db.execute(
        "UPDATE products SET category_id = 1 WHERE category = 'Electronics'",
        (),
    )
    .unwrap();
    db.execute(
        "UPDATE products SET category_id = 2 WHERE category = 'Accessories'",
        (),
    )
    .unwrap();

    // 4. Add foreign key constraint
    db.execute("ALTER TABLE products ADD CONSTRAINT fk_category FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL", ()).unwrap();

    // 5. Transaction with partial insert
    let mut tx = db.begin().unwrap();
    tx.execute(
        "UPDATE products SET price = price * 0.9 WHERE category = 'Electronics'",
        (),
    )
    .unwrap();

    // Notice that category_id is missing in this INSERT statement!
    // It should be padded with NULL and should not fail with "invalid column count"
    tx.execute(
        "INSERT INTO products (id, name, description, price, category, in_stock, created_at) VALUES (6, 'Tablet', '12.9-inch iPad Pro', 999.99, 'Electronics', TRUE, NOW())",
        ()
    ).unwrap();

    tx.commit().unwrap();

    // Verify row 6 was added successfully
    let mut select = db
        .query(
            "SELECT id, name, category_id FROM products WHERE id = 6",
            (),
        )
        .unwrap();
    let row = select.next().unwrap().unwrap();
    assert_eq!(row.get_value(0).unwrap(), &Value::Integer(6));
    assert_eq!(row.get_value(1).unwrap(), &Value::text("Tablet"));
    assert_eq!(row.get_value(2).unwrap(), &Value::null_unknown()); // Padded as NULL
}
