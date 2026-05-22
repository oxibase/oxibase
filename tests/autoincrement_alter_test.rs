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
fn test_autoincrement_alter() {
    let db = Database::open_in_memory().unwrap();

    // 1. Create a table without AUTOINCREMENT
    db.execute(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            name TEXT
        )",
        (),
    )
    .unwrap();

    // 2. Insert a record with an explicit ID
    db.execute("INSERT INTO products (id, name) VALUES (10, 'Laptop')", ())
        .unwrap();

    // 3. Alter the table to add AUTOINCREMENT
    db.execute(
        "ALTER TABLE products MODIFY COLUMN id INTEGER AUTOINCREMENT",
        (),
    )
    .unwrap();

    // 4. Insert records without specifying an ID
    db.execute("INSERT INTO products (name) VALUES ('Mouse')", ())
        .unwrap();
    db.execute("INSERT INTO products (name) VALUES ('Keyboard')", ())
        .unwrap();

    // 5. Verify the generated IDs
    let mut result = db
        .query("SELECT id, name FROM products ORDER BY id", ())
        .unwrap();

    let row1 = result.next().unwrap().unwrap();
    assert_eq!(row1.get::<i64>(0).unwrap(), 1);
    assert_eq!(row1.get::<String>(1).unwrap(), "Mouse");

    let row2 = result.next().unwrap().unwrap();
    assert_eq!(row2.get::<i64>(0).unwrap(), 2);
    assert_eq!(row2.get::<String>(1).unwrap(), "Keyboard");

    let row3 = result.next().unwrap().unwrap();
    assert_eq!(row3.get::<i64>(0).unwrap(), 10);
    assert_eq!(row3.get::<String>(1).unwrap(), "Laptop");

    assert!(result.next().is_none());

    // 6. Test failure if adding AUTOINCREMENT to non-integer column
    let err = db.execute(
        "ALTER TABLE products MODIFY COLUMN name TEXT AUTOINCREMENT",
        (),
    );
    assert!(err.is_err());
    assert!(matches!(err.unwrap_err(), Error::InvalidArgumentMessage(_)));
}
