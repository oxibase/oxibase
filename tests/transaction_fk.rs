// Copyright 2026 Oxibase Contributors
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

use oxibase::api::database::Database;

#[test]
fn test_transaction_rollback_fk() {
    let db = Database::open_in_memory().unwrap();

    db.execute(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            price FLOAT NOT NULL,
            category TEXT
        );",
        (),
    )
    .unwrap();

    db.execute(
        "INSERT INTO products (id, name, price, category) VALUES 
         (1, 'Laptop', 1000.0, 'Electronics');",
        (),
    )
    .unwrap();

    // Start a transaction, update, and rollback
    let mut txn = db.begin().unwrap();
    txn.execute(
        "UPDATE products SET price = 900.0 WHERE category = 'Electronics';",
        (),
    )
    .unwrap();
    txn.rollback().unwrap();

    // The rollback should have released row locks. Let's try adding a foreign key now.
    db.execute(
        "CREATE TABLE categories (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );",
        (),
    )
    .unwrap();

    db.execute(
        "INSERT INTO categories (id, name) VALUES (1, 'Electronics');",
        (),
    )
    .unwrap();
    db.execute("ALTER TABLE products ADD COLUMN category_id INTEGER;", ())
        .unwrap();
    db.execute(
        "UPDATE products SET category_id = 1 WHERE category = 'Electronics';",
        (),
    )
    .unwrap();

    // This should NOT fail with "row has uncommitted changes"
    let res = db.execute("ALTER TABLE products ADD CONSTRAINT fk_category FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL;", ());
    assert!(
        res.is_ok(),
        "Expected constraint addition to succeed, but got: {:?}",
        res.err()
    );
}

#[test]
fn test_transaction_commit_fk() {
    let db = Database::open_in_memory().unwrap();

    db.execute(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            price FLOAT NOT NULL,
            category TEXT
        );",
        (),
    )
    .unwrap();

    db.execute(
        "INSERT INTO products (id, name, price, category) VALUES 
         (1, 'Laptop', 1000.0, 'Electronics');",
        (),
    )
    .unwrap();

    db.execute(
        "CREATE TABLE categories (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );",
        (),
    )
    .unwrap();

    db.execute(
        "INSERT INTO categories (id, name) VALUES (1, 'Electronics');",
        (),
    )
    .unwrap();
    db.execute("ALTER TABLE products ADD COLUMN category_id INTEGER;", ())
        .unwrap();
    db.execute(
        "UPDATE products SET category_id = 1 WHERE category = 'Electronics';",
        (),
    )
    .unwrap();
    db.execute("ALTER TABLE products ADD CONSTRAINT fk_category FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL;", ()).unwrap();

    // Start a transaction, update referenced row, and commit
    let mut txn = db.begin().unwrap();
    txn.execute(
        "UPDATE products SET price = 850.0 WHERE category = 'Electronics';",
        (),
    )
    .unwrap();
    let res = txn.commit();

    // The commit should succeed without "row has uncommitted changes"
    assert!(
        res.is_ok(),
        "Expected commit to succeed, but got: {:?}",
        res.err()
    );
}
