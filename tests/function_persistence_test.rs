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

//! Tests for user-defined function persistence
//!
//! These tests verify that user-defined functions are properly persisted
//! to and loaded from the system table across database restarts.

#[cfg(feature = "boa")]
use oxibase::Database;

/// Test basic function persistence and loading
#[cfg(feature = "boa")]
#[test]
fn test_function_persistence_basic() {
    let db = Database::open("memory://test").unwrap();

    // Create a simple function
    db.execute(
        "CREATE FUNCTION test_add(a INTEGER, b INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return a + b;'",
        (),
    ).expect("Failed to create function");

    // First verify the function works
    let call_result: i64 = db
        .query_one("SELECT test_add(5, 3)", ())
        .expect("Failed to call function");
    assert_eq!(call_result, 8);

    // Verify function is listed in SHOW FUNCTIONS
    let result = db
        .query("SHOW FUNCTIONS", ())
        .expect("Failed to execute SHOW FUNCTIONS");
    let rows: Vec<_> = result.collect();
    assert_eq!(rows.len(), 1);

    // Check the function name
    if let Some(Ok(row)) = rows.first() {
        let name: String = row.get(0).expect("Failed to get name");
        assert_eq!(name, "TEST_ADD");
    }

    // Verify the function can be called
    let result: i64 = db
        .query_one("SELECT test_add(5, 3)", ())
        .expect("Failed to call function");

    assert_eq!(result, 8);
}

#[cfg(feature = "boa")]
#[test]
fn test_function_persistence_restart() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir
        .path()
        .join("test.db")
        .to_str()
        .unwrap()
        .to_string();

    // First session: create function and persist it
    {
        let db = Database::open(&format!("file://{}", db_path)).expect("Failed to open database");

        // Create a function
        db.execute(
            "CREATE FUNCTION persistent_func(x INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return x * 2;'",
            (),
        ).expect("Failed to create function");

        // Use the function to verify it works
        let result: i64 = db
            .query_one("SELECT persistent_func(5)", ())
            .expect("Failed to call function");
        assert_eq!(result, 10);
    }

    // Second session: create new database connection and verify function is loaded
    {
        let db = Database::open(&format!("file://{}", db_path)).expect("Failed to reopen database");

        // Function should work after restart (proves it was loaded from system table)
        let result: i64 = db
            .query_one("SELECT persistent_func(7)", ())
            .expect("Failed to call function after restart");
        assert_eq!(result, 14);
    }
}

/// Test multiple functions persistence
#[cfg(feature = "boa")]
#[test]
fn test_multiple_functions_persistence() {
    let db = Database::open("memory://multi_func").expect("Failed to create database");

    // Create multiple functions
    db.execute(
        "CREATE FUNCTION func1(a TEXT) RETURNS TEXT LANGUAGE DENO AS 'return `Hello ${a}`;'",
        (),
    )
    .expect("Failed to create func1");

    db.execute(
        "CREATE FUNCTION func2(x INTEGER, y INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return Math.max(x, y);'",
        (),
    ).expect("Failed to create func2");

    // Verify both functions are persisted
    let count: i64 = db
        .query_one("SELECT COUNT(*) FROM _sys_functions", ())
        .expect("Failed to count functions");
    assert_eq!(count, 2);

    // Test both functions work
    let result1: String = db
        .query_one("SELECT func1('World')", ())
        .expect("Failed to call func1");
    assert_eq!(result1, "Hello World");

    let result2: i64 = db
        .query_one("SELECT func2(3, 7)", ())
        .expect("Failed to call func2");
    assert_eq!(result2, 7);
}

/// Test CREATE FUNCTION IF NOT EXISTS with persistence
#[cfg(feature = "boa")]
#[test]
fn test_function_if_not_exists_persistence() {
    let db = Database::open("memory://if_not_exists").expect("Failed to create database");

    // Create function first time
    db.execute(
        "CREATE FUNCTION IF NOT EXISTS conditional_func() RETURNS TEXT LANGUAGE DENO AS 'return \"created\";'",
        (),
    ).expect("Failed to create function first time");

    // Try to create again - should not fail
    db.execute(
        "CREATE FUNCTION IF NOT EXISTS conditional_func() RETURNS TEXT LANGUAGE DENO AS 'return \"duplicate\";'",
        (),
    ).expect("Failed to create function second time");

    // Function should still exist and work with original implementation
    let result: String = db
        .query_one("SELECT conditional_func()", ())
        .expect("Failed to call function");
    assert_eq!(result, "created");
}

/// Test that system table starts empty (created on first function)
#[cfg(feature = "boa")]
#[test]
fn test_functions_table_starts_empty() {
    let db = Database::open("memory://empty_table").expect("Failed to create database");

    // System table doesn't exist yet - this should fail
    let result = db.query_one::<i64, _>("SELECT COUNT(*) FROM _sys_functions", ());
    assert!(result.is_err(), "System table should not exist initially");

    // Create a function to trigger table creation
    db.execute(
        "CREATE FUNCTION temp_func() RETURNS INTEGER LANGUAGE DENO AS 'return 42;'",
        (),
    )
    .expect("Failed to create function");

    // Now the table should exist and have 1 row
    let count: i64 = db
        .query_one("SELECT COUNT(*) FROM _sys_functions", ())
        .expect("Failed to count functions");
    assert_eq!(count, 1);
}

#[cfg(feature = "boa")]
#[test]
fn test_show_functions() {
    let db = Database::open("memory://show_functions").expect("Failed to create database");

    // Initially no functions
    let result = db
        .query("SHOW FUNCTIONS", ())
        .expect("Failed to execute SHOW FUNCTIONS");
    let rows: Vec<_> = result.collect();
    assert_eq!(rows.len(), 0);

    // Create some functions
    db.execute(
        "CREATE FUNCTION add_nums(a INTEGER, b INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return a + b;'",
        (),
    )
    .expect("Failed to create function");

    let result = db
        .query("SELECT name FROM _sys_functions", ())
        .expect("Failed to select functions after first");
    let rows: Vec<_> = result.map(|r| r.unwrap()).collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String>(0).unwrap(), "ADD_NUMS");

    db.execute(
        "CREATE FUNCTION greet(name TEXT) RETURNS TEXT LANGUAGE DENO AS 'return `Hello, ${name}!`;'",
        (),
    )
    .expect("Failed to create function");

    // Check that functions are persisted
    let count: i64 = db
        .query_one("SELECT COUNT(*) FROM _sys_functions", ())
        .expect("Failed to count functions");
    assert_eq!(count, 2);

    // Now SHOW FUNCTIONS should return them
    let result = db
        .query("SHOW FUNCTIONS", ())
        .expect("Failed to execute SHOW FUNCTIONS");
    let rows: Vec<_> = result.map(|r| r.unwrap()).collect();
    assert_eq!(rows.len(), 2);

    // Check first function (ADD_NUMS)
    let row = &rows[0];
    assert_eq!(row.get::<String>(0).unwrap(), "ADD_NUMS");
    assert_eq!(row.get::<String>(1).unwrap(), "(a INTEGER, b INTEGER)");
    assert_eq!(row.get::<String>(2).unwrap(), "INTEGER");
    assert_eq!(row.get::<String>(3).unwrap(), "DENO");
    assert!(row.get::<String>(4).unwrap().contains("a + b"));

    // Check second function (GREET)
    let row = &rows[1];
    assert_eq!(row.get::<String>(0).unwrap(), "GREET");
    assert_eq!(row.get::<String>(1).unwrap(), "(name TEXT)");
    assert_eq!(row.get::<String>(2).unwrap(), "TEXT");
    assert_eq!(row.get::<String>(3).unwrap(), "DENO");
    assert!(row.get::<String>(4).unwrap().contains("Hello"));
}

/// Test basic DROP FUNCTION functionality
#[cfg(feature = "boa")]
#[test]
fn test_drop_function_basic() {
    let db = Database::open("memory://drop_basic").expect("Failed to create database");

    // Create a function
    db.execute(
        "CREATE FUNCTION drop_me(x INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return x * 3;'",
        (),
    )
    .expect("Failed to create function");

    // Verify function exists and works
    let result: i64 = db
        .query_one("SELECT drop_me(4)", ())
        .expect("Failed to call function");
    assert_eq!(result, 12);

    // Verify it's in system table
    let count: i64 = db
        .query_one(
            "SELECT COUNT(*) FROM _sys_functions WHERE name = 'DROP_ME'",
            (),
        )
        .expect("Failed to count functions");
    assert_eq!(count, 1);

    // Drop the function
    db.execute("DROP FUNCTION drop_me", ())
        .expect("Failed to drop function");

    // Verify it's removed from system table
    let count: i64 = db
        .query_one(
            "SELECT COUNT(*) FROM _sys_functions WHERE name = 'DROP_ME'",
            (),
        )
        .expect("Failed to count functions after drop");
    assert_eq!(count, 0);

    // Verify function can no longer be called
    let result = db.query_one::<i64, _>("SELECT drop_me(4)", ());
    assert!(result.is_err(), "Function should not exist after DROP");
}

/// Test DROP FUNCTION IF EXISTS when function exists
#[cfg(feature = "boa")]
#[test]
fn test_drop_function_if_exists_exists() {
    let db = Database::open("memory://drop_if_exists").expect("Failed to create database");

    // Create a function
    db.execute(
        "CREATE FUNCTION if_exists_func() RETURNS TEXT LANGUAGE DENO AS 'return \"exists\";'",
        (),
    )
    .expect("Failed to create function");

    // Drop with IF EXISTS (should succeed)
    db.execute("DROP FUNCTION IF EXISTS if_exists_func", ())
        .expect("Failed to drop function with IF EXISTS");

    // Verify it's gone
    let count: i64 = db
        .query_one(
            "SELECT COUNT(*) FROM _sys_functions WHERE name = 'IF_EXISTS_FUNC'",
            (),
        )
        .expect("Failed to count functions");
    assert_eq!(count, 0);
}

/// Test DROP FUNCTION IF EXISTS when function doesn't exist
#[cfg(feature = "boa")]
#[test]
fn test_drop_function_if_exists_not_exists() {
    let db = Database::open("memory://drop_if_not_exists").expect("Failed to create database");

    // Drop non-existent function with IF EXISTS (should succeed without error)
    db.execute("DROP FUNCTION IF EXISTS nonexistent_func", ())
        .expect("DROP IF EXISTS should not fail for non-existent function");

    // Verify no functions exist (table might not exist yet)
    let count_result: Result<i64, _> = db.query_one("SELECT COUNT(*) FROM _sys_functions", ());
    match count_result {
        Ok(count) => assert_eq!(count, 0),
        Err(_) => {
            // Table doesn't exist, which is fine - no functions exist
        }
    }
}

/// Test DROP FUNCTION on non-existent function without IF EXISTS
#[cfg(feature = "boa")]
#[test]
fn test_drop_function_not_exists_error() {
    let db = Database::open("memory://drop_error").expect("Failed to create database");

    // Try to drop non-existent function without IF EXISTS (should fail)
    let result = db.execute("DROP FUNCTION nonexistent_func", ());
    assert!(
        result.is_err(),
        "DROP without IF EXISTS should fail for non-existent function"
    );
}

/// Test DROP FUNCTION removes function from registry
#[cfg(feature = "boa")]
#[test]
fn test_drop_function_registry_cleanup() {
    let db = Database::open("memory://drop_registry").expect("Failed to create database");

    // Create multiple functions
    db.execute(
        "CREATE FUNCTION keep_func() RETURNS INTEGER LANGUAGE DENO AS 'return 1;'",
        (),
    )
    .expect("Failed to create keep_func");

    db.execute(
        "CREATE FUNCTION remove_func() RETURNS INTEGER LANGUAGE DENO AS 'return 2;'",
        (),
    )
    .expect("Failed to create remove_func");

    // Verify both work
    let result1: i64 = db
        .query_one("SELECT keep_func()", ())
        .expect("Failed to call keep_func");
    assert_eq!(result1, 1);

    let result2: i64 = db
        .query_one("SELECT remove_func()", ())
        .expect("Failed to call remove_func");
    assert_eq!(result2, 2);

    // Drop one function
    db.execute("DROP FUNCTION remove_func", ())
        .expect("Failed to drop remove_func");

    // Verify keep_func still works
    let result1: i64 = db
        .query_one("SELECT keep_func()", ())
        .expect("keep_func should still work");
    assert_eq!(result1, 1);

    // Verify remove_func is gone from registry (can't be called)
    let result = db.query_one::<i64, _>("SELECT remove_func()", ());
    assert!(
        result.is_err(),
        "remove_func should not be callable after DROP"
    );

    // Verify only one function remains in system table
    let count: i64 = db
        .query_one("SELECT COUNT(*) FROM _sys_functions", ())
        .expect("Failed to count functions");
    assert_eq!(count, 1);

    let name: String = db
        .query_one("SELECT name FROM _sys_functions", ())
        .expect("Failed to get remaining function name");
    assert_eq!(name, "KEEP_FUNC");
}

/// Test DROP FUNCTION persistence across restart
#[cfg(feature = "boa")]
#[test]
fn test_drop_function_persistence_restart() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir
        .path()
        .join("test_drop.db")
        .to_str()
        .unwrap()
        .to_string();

    // First session: create function, then drop it
    {
        let db = Database::open(&format!("file://{}", db_path)).expect("Failed to open database");

        // Create a function
        db.execute(
            "CREATE FUNCTION temp_drop_func(x INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return x + 100;'",
            (),
        ).expect("Failed to create function");

        // Verify it works
        let result: i64 = db
            .query_one("SELECT temp_drop_func(5)", ())
            .expect("Failed to call function");
        assert_eq!(result, 105);

        // Drop the function
        db.execute("DROP FUNCTION temp_drop_func", ())
            .expect("Failed to drop function");
    }

    // Second session: verify function stays dropped after restart
    {
        let db = Database::open(&format!("file://{}", db_path)).expect("Failed to reopen database");

        // Function should not exist after restart (proves DROP was persisted)
        let result = db.query_one::<i64, _>("SELECT temp_drop_func(5)", ());
        assert!(
            result.is_err(),
            "Function should remain dropped after restart"
        );

        // System table should be empty or not have the function
        let count: i64 = db
            .query_one(
                "SELECT COUNT(*) FROM _sys_functions WHERE name = 'TEMP_DROP_FUNC'",
                (),
            )
            .expect("Failed to count functions after restart");
        assert_eq!(count, 0);
    }
}

/// Test CREATE FUNCTION with schema-qualified names
#[cfg(feature = "boa")]
#[test]
fn test_create_function_schema_qualified() {
    let db = Database::open("memory://create_schema_func").expect("Failed to create database");

    // Create function with schema-qualified name
    db.execute(
        "CREATE FUNCTION myschema.add_nums(a INTEGER, b INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return a + b;'",
        (),
    ).expect("Failed to create schema-qualified function");

    // Verify function exists and works
    let result: i64 = db
        .query_one("SELECT myschema.add_nums(3, 4)", ())
        .expect("Failed to call schema-qualified function");
    assert_eq!(result, 7);

    // Verify it's stored with schema
    let schema: Option<String> = db
        .query_one(
            "SELECT schema FROM _sys_functions WHERE name = 'ADD_NUMS'",
            (),
        )
        .expect("Failed to query schema");
    assert_eq!(schema, Some("MYSCHEMA".to_string()));

    let name: String = db
        .query_one(
            "SELECT name FROM _sys_functions WHERE name = 'ADD_NUMS'",
            (),
        )
        .expect("Failed to query name");
    assert_eq!(name, "ADD_NUMS");
}

/// Test DROP FUNCTION with schema-qualified names
#[cfg(feature = "boa")]
#[test]
fn test_drop_function_schema_qualified() {
    let db = Database::open("memory://drop_schema_func").expect("Failed to create database");

    // Create function with schema-qualified name
    db.execute(
        "CREATE FUNCTION test_schema.multiply(x INTEGER, y INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return x * y;'",
        (),
    ).expect("Failed to create schema-qualified function");

    // Verify it works
    let result: i64 = db
        .query_one("SELECT test_schema.multiply(5, 6)", ())
        .expect("Failed to call function");
    assert_eq!(result, 30);

    // Drop with schema-qualified name
    db.execute("DROP FUNCTION test_schema.multiply", ())
        .expect("Failed to drop schema-qualified function");

    // Verify function is gone
    let result = db.query_one::<i64, _>("SELECT test_schema.multiply(5, 6)", ());
    assert!(result.is_err(), "Function should not exist after DROP");

    // Verify system table is empty
    let count: i64 = db
        .query_one("SELECT COUNT(*) FROM _sys_functions", ())
        .expect("Failed to count functions");
    assert_eq!(count, 0);
}

/// Test function invocation with schema-qualified names
#[cfg(feature = "boa")]
#[test]
fn test_function_call_schema_qualified() {
    let db = Database::open("memory://call_schema_func").expect("Failed to create database");

    // Create function with schema-qualified name
    db.execute(
        "CREATE FUNCTION db.calc_power(base INTEGER, exp INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return Math.pow(base, exp);'",
        (),
    ).expect("Failed to create function");

    // Call with schema qualification
    let result: f64 = db
        .query_one("SELECT db.calc_power(2, 3)", ())
        .expect("Failed to call with schema qualification");
    assert_eq!(result, 8.0);

    // Call without schema qualification (should still work since functions are global)
    let result2: f64 = db
        .query_one("SELECT calc_power(3, 2)", ())
        .expect("Failed to call without schema qualification");
    assert_eq!(result2, 9.0);
}

/// Test that functions remain global despite schema qualification
#[cfg(feature = "boa")]
#[test]
fn test_functions_remain_global() {
    let db = Database::open("memory://global_funcs").expect("Failed to create database");

    // Create function with different schema qualifications
    db.execute(
        "CREATE FUNCTION schema1.func() RETURNS TEXT LANGUAGE DENO AS 'return \"schema1\";'",
        (),
    )
    .expect("Failed to create function with schema1");

    // Try to create function with same name but different schema (should fail)
    let result = db.execute(
        "CREATE FUNCTION schema2.func() RETURNS TEXT LANGUAGE DENO AS 'return \"schema2\";'",
        (),
    );
    assert!(
        result.is_err(),
        "Should not allow duplicate function names even with different schemas"
    );

    // Function should be accessible with any schema qualification
    let result1: String = db
        .query_one("SELECT schema1.func()", ())
        .expect("Failed to call with original schema");
    assert_eq!(result1, "schema1");

    let result2: String = db
        .query_one("SELECT schema2.func()", ())
        .expect("Failed to call with different schema");
    assert_eq!(result2, "schema1"); // Should return the original function

    let result3: String = db
        .query_one("SELECT func()", ())
        .expect("Failed to call without schema");
    assert_eq!(result3, "schema1"); // Should work without schema
}
