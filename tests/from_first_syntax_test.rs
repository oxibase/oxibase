// Copyright 2025 Stoolap Contributors
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

use oxibase::Database;

#[tokio::test]
async fn test_from_first_syntax() {
    let db = Database::open("memory://test_from_first").unwrap();

    // Setup table
    db.execute(
        "CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)",
        (),
    )
    .unwrap();
    db.execute(
        "INSERT INTO users VALUES (1, 'Alice', 30), (2, 'Bob', 25), (3, 'Charlie', 35)",
        (),
    )
    .unwrap();

    // Test 1: FROM ... SELECT ...
    let rows = db
        .query("FROM users SELECT id, name", ())
        .unwrap()
        .collect_vec()
        .unwrap();
    assert_eq!(rows.len(), 3);
    let mut names: Vec<String> = rows
        .into_iter()
        .map(|r| r.get::<String>(1).unwrap())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);

    // Test 2: FROM ... WHERE ... SELECT ...
    let rows = db
        .query("FROM users WHERE age > 25 SELECT name", ())
        .unwrap()
        .collect_vec()
        .unwrap();
    assert_eq!(rows.len(), 2);
    let mut names: Vec<String> = rows
        .into_iter()
        .map(|r| r.get::<String>(0).unwrap())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Alice", "Charlie"]);

    // Test 3: FROM ... (implicit SELECT *)
    let rows_iter = db.query("FROM users", ()).unwrap();
    assert_eq!(rows_iter.column_count(), 3);
    let rows = rows_iter.collect_vec().unwrap();
    assert_eq!(rows.len(), 3);

    // Test 4: FROM ... ORDER BY ... LIMIT ...
    let rows = db
        .query("FROM users ORDER BY age DESC LIMIT 1", ())
        .unwrap()
        .collect_vec()
        .unwrap();
    assert_eq!(rows.len(), 1);
    let name: String = rows[0].get(1).unwrap();
    assert_eq!(name, "Charlie");
}
