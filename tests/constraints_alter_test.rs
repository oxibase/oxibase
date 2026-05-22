// Copyright 2025 Oxibase Contributors
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

use oxibase::core::Error;
use oxibase::Database;

#[test]
fn test_unique_constraint_alter() {
    let db = Database::open_in_memory().unwrap();

    db.execute(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            email TEXT
        )",
        (),
    )
    .unwrap();

    // 1. Insert records
    db.execute(
        "INSERT INTO users (id, email) VALUES (1, 'test@example.com')",
        (),
    )
    .unwrap();

    // 2. Alter the table to add UNIQUE constraint
    db.execute("ALTER TABLE users MODIFY COLUMN email TEXT UNIQUE", ())
        .unwrap();

    // 3. Insert record with duplicate email should fail
    let err = db.execute(
        "INSERT INTO users (id, email) VALUES (2, 'test@example.com')",
        (),
    );
    assert!(err.is_err());
    assert!(matches!(err.unwrap_err(), Error::UniqueConstraint { .. }));

    // 4. Insert record with new email should succeed
    db.execute(
        "INSERT INTO users (id, email) VALUES (2, 'test2@example.com')",
        (),
    )
    .unwrap();

    // 5. Verify the generated IDs
    let mut result = db
        .query("SELECT id, email FROM users ORDER BY id", ())
        .unwrap();

    let row1 = result.next().unwrap().unwrap();
    assert_eq!(row1.get::<i64>(0).unwrap(), 1);

    let row2 = result.next().unwrap().unwrap();
    assert_eq!(row2.get::<i64>(0).unwrap(), 2);

    assert!(result.next().is_none());
}

#[test]
fn test_check_constraint_alter() {
    let db = Database::open_in_memory().unwrap();

    db.execute(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            price FLOAT
        )",
        (),
    )
    .unwrap();

    // 1. Insert record
    db.execute("INSERT INTO products (id, price) VALUES (1, 10.5)", ())
        .unwrap();

    // 2. Alter the table to add CHECK constraint
    db.execute(
        "ALTER TABLE products MODIFY COLUMN price FLOAT CHECK (price > 0)",
        (),
    )
    .unwrap();

    // 3. Insert record with invalid price should fail
    let err = db.execute("INSERT INTO products (id, price) VALUES (2, -5.0)", ());
    assert!(err.is_err());
    assert!(matches!(
        err.unwrap_err(),
        Error::CheckConstraintViolation { .. }
    ));

    // 4. Insert record with valid price should succeed
    db.execute("INSERT INTO products (id, price) VALUES (2, 20.0)", ())
        .unwrap();

    // 5. Verify
    let mut result = db
        .query("SELECT id, price FROM products ORDER BY id", ())
        .unwrap();

    let row1 = result.next().unwrap().unwrap();
    assert_eq!(row1.get::<i64>(0).unwrap(), 1);

    let row2 = result.next().unwrap().unwrap();
    assert_eq!(row2.get::<i64>(0).unwrap(), 2);

    assert!(result.next().is_none());
}
