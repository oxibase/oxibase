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
// See the License for the specific language governing permissions and limitations
// under the License.

//! Tests for information_schema virtual tables

use oxibase::api::Database;

#[test]
fn test_information_schema_tables() {
    let db = Database::open("memory://info_schema_tables").expect("Failed to create database");

    // Create some tables and views
    db.execute("CREATE TABLE users (id INTEGER, name TEXT)", ())
        .expect("Failed to create table");
    db.execute("CREATE TABLE products (id INTEGER, price FLOAT)", ())
        .expect("Failed to create table");
    db.execute(
        "CREATE VIEW active_users AS SELECT * FROM users WHERE id > 0",
        (),
    )
    .expect("Failed to create view");

    // Query information_schema.tables
    let result = db
        .query(
            "SELECT * FROM \"information_schema\".\"tables\" ORDER BY table_name",
            (),
        )
        .expect("Failed to query information_schema.tables");

    let mut tables = Vec::new();
    for row in result {
        let row = row.expect("Failed to read row");
        let table_name: String = row.get(2).expect("Failed to get table_name");
        let table_type: String = row.get(3).expect("Failed to get table_type");
        tables.push((table_name, table_type));
    }

    assert_eq!(tables.len(), 3);
    assert_eq!(tables[0], ("active_users".to_string(), "VIEW".to_string()));
    assert_eq!(
        tables[1],
        ("products".to_string(), "BASE TABLE".to_string())
    );
    assert_eq!(tables[2], ("users".to_string(), "BASE TABLE".to_string()));
}

#[test]
fn test_information_schema_unquoted_qualified_identifiers() {
    let db = Database::open("memory://info_schema_unquoted").expect("Failed to create database");

    // Create some tables and views
    db.execute("CREATE TABLE users (id INTEGER, name TEXT)", ())
        .expect("Failed to create table");
    db.execute("CREATE TABLE products (id INTEGER, price FLOAT)", ())
        .expect("Failed to create table");
    db.execute(
        "CREATE VIEW active_users AS SELECT * FROM users WHERE id > 0",
        (),
    )
    .expect("Failed to create view");

    // Query information_schema.tables with unquoted qualified identifiers
    let result = db
        .query(
            "SELECT * FROM information_schema.tables ORDER BY table_name",
            (),
        )
        .expect("Failed to query information_schema.tables with unquoted identifiers");

    let mut tables = Vec::new();
    for row in result {
        let row = row.expect("Failed to read row");
        let table_name: String = row.get(2).expect("Failed to get table_name");
        let table_type: String = row.get(3).expect("Failed to get table_type");
        tables.push((table_name, table_type));
    }

    assert_eq!(tables.len(), 3);
    assert_eq!(tables[0], ("active_users".to_string(), "VIEW".to_string()));
    assert_eq!(
        tables[1],
        ("products".to_string(), "BASE TABLE".to_string())
    );
    assert_eq!(tables[2], ("users".to_string(), "BASE TABLE".to_string()));

    // Also test other information_schema tables with unquoted identifiers
    let result = db
        .query("SELECT * FROM information_schema.columns WHERE table_name = 'users' ORDER BY column_name", ())
        .expect("Failed to query information_schema.columns with unquoted identifiers");

    let mut columns = Vec::new();
    for row in result {
        let row = row.expect("Failed to read row");
        let column_name: String = row.get(3).expect("Failed to get column_name");
        columns.push(column_name);
    }

    assert_eq!(columns.len(), 2);
    assert_eq!(columns[0], "id");
    assert_eq!(columns[1], "name");
}

#[test]
fn test_information_schema_columns() {
    let db = Database::open("memory://info_schema_columns").expect("Failed to create database");

    // Create a table with various column types
    db.execute(
        "CREATE TABLE test_table (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER DEFAULT 0,
            price FLOAT
        )",
        (),
    )
    .expect("Failed to create table");

    // Query information_schema.columns
    let result = db
        .query(
            "SELECT column_name, data_type, is_nullable, column_default
             FROM \"information_schema\".\"columns\"
             WHERE table_name = 'test_table'
             ORDER BY ordinal_position",
            (),
        )
        .expect("Failed to query information_schema.columns");

    let mut columns = Vec::new();
    for row in result {
        let row = row.expect("Failed to read row");
        let col_name: String = row.get(0).expect("Failed to get column_name");
        let data_type: String = row.get(1).expect("Failed to get data_type");
        let is_nullable: String = row.get(2).expect("Failed to get is_nullable");
        let default: Option<String> = row.get(3).ok();
        columns.push((col_name, data_type, is_nullable, default));
    }

    assert_eq!(columns.len(), 4);
    assert_eq!(
        columns[0],
        (
            "id".to_string(),
            "Integer".to_string(),
            "NO".to_string(),
            Some("".to_string())
        )
    );
    assert_eq!(
        columns[1],
        (
            "name".to_string(),
            "Text".to_string(),
            "NO".to_string(),
            Some("".to_string())
        )
    );
    assert_eq!(
        columns[2],
        (
            "age".to_string(),
            "Integer".to_string(),
            "YES".to_string(),
            Some("0".to_string())
        )
    );
    assert_eq!(
        columns[3],
        (
            "price".to_string(),
            "Float".to_string(),
            "YES".to_string(),
            Some("".to_string())
        )
    );
}

#[test]
fn test_information_schema_functions() {
    let db = Database::open("memory://info_schema_functions").expect("Failed to create database");

    // Query information_schema.functions
    let result = db
        .query(
            "SELECT function_name, function_type, data_type
             FROM \"information_schema\".\"functions\"
             WHERE function_name IN ('UPPER', 'COUNT', 'ROW_NUMBER')
             ORDER BY function_name",
            (),
        )
        .expect("Failed to query information_schema.functions");

    let mut functions = Vec::new();
    for row in result {
        let row = row.expect("Failed to read row");
        let func_name: String = row.get(0).expect("Failed to get function_name");
        let func_type: String = row.get(1).expect("Failed to get function_type");
        let data_type: String = row.get(2).expect("Failed to get data_type");
        functions.push((func_name, func_type, data_type));
    }

    // Check that we have the expected functions
    assert!(functions.len() >= 3);
    let upper = functions
        .iter()
        .find(|(name, _, _)| name == "UPPER")
        .unwrap();
    assert_eq!(upper.1, "SCALAR");
    assert_eq!(upper.2, "TEXT");

    let count = functions
        .iter()
        .find(|(name, _, _)| name == "COUNT")
        .unwrap();
    assert_eq!(count.1, "AGGREGATE");
    assert_eq!(count.2, "INTEGER");

    let row_number = functions
        .iter()
        .find(|(name, _, _)| name == "ROW_NUMBER")
        .unwrap();
    assert_eq!(row_number.1, "WINDOW");
    assert_eq!(row_number.2, "INTEGER");
}

#[cfg(feature = "boa")]
#[test]
fn test_information_schema_functions_user_defined() {
    let db = Database::open("memory://info_schema_udf").expect("Failed to create database");

    // Create user-defined functions
    db.execute(
        "CREATE FUNCTION greet(name TEXT) RETURNS TEXT LANGUAGE DENO AS 'return `Hello, ${arguments[0]}!`;'",
        (),
    )
    .expect("Failed to create greet function");

    db.execute(
        "CREATE FUNCTION square(x INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return arguments[0] * arguments[0];'",
        (),
    )
    .expect("Failed to create square function");

    db.execute(
        "CREATE FUNCTION is_even(n INTEGER) RETURNS BOOLEAN LANGUAGE DENO AS 'return arguments[0] % 2 === 0;'",
        (),
    )
    .expect("Failed to create is_even function");

    db.execute(
        "CREATE FUNCTION create_person(name TEXT, age INTEGER) RETURNS JSON LANGUAGE DENO AS 'return { name: arguments[0], age: arguments[1] };'",
        (),
    )
    .expect("Failed to create create_person function");

    // Query information_schema.functions for user-defined functions
    let result = db
        .query(
            "SELECT function_name, function_type, data_type, function_schema
             FROM information_schema.functions
             WHERE function_schema != 'sys'
             ORDER BY function_name",
            (),
        )
        .expect("Failed to query information_schema.functions");

    let mut functions = Vec::new();
    for row in result {
        let row = row.expect("Failed to read row");
        let func_name: String = row.get(0).expect("Failed to get function_name");
        let func_type: String = row.get(1).expect("Failed to get function_type");
        let data_type: String = row.get(2).expect("Failed to get data_type");
        let schema: Option<String> = row.get(3).expect("Failed to get function_schema");
        functions.push((func_name, func_type, data_type, schema));
    }

    // Check that we have the expected user-defined functions
    assert_eq!(functions.len(), 4);

    let create_person = functions
        .iter()
        .find(|(name, _, _, _)| name == "CREATE_PERSON")
        .unwrap();
    assert_eq!(create_person.1, "SCALAR");
    assert_eq!(create_person.2, "JSON");
    assert_eq!(create_person.3, Some("public".to_string()));

    let greet = functions
        .iter()
        .find(|(name, _, _, _)| name == "GREET")
        .unwrap();
    assert_eq!(greet.1, "SCALAR");
    assert_eq!(greet.2, "TEXT");
    assert_eq!(greet.3, Some("public".to_string()));

    let is_even = functions
        .iter()
        .find(|(name, _, _, _)| name == "IS_EVEN")
        .unwrap();
    assert_eq!(is_even.1, "SCALAR");
    assert_eq!(is_even.2, "BOOLEAN");
    assert_eq!(is_even.3, Some("public".to_string()));

    let square = functions
        .iter()
        .find(|(name, _, _, _)| name == "SQUARE")
        .unwrap();
    assert_eq!(square.1, "SCALAR");
    assert_eq!(square.2, "INTEGER");
    assert_eq!(square.3, Some("public".to_string()));
}

#[test]
fn test_information_schema_statistics() {
    let db = Database::open("memory://info_schema_stats").expect("Failed to create database");

    // Create a table with indexes
    db.execute(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
        (),
    )
    .expect("Failed to create table");
    db.execute("CREATE INDEX idx_name ON users (name)", ())
        .expect("Failed to create index");
    db.execute("CREATE INDEX idx_email ON users (email)", ())
        .expect("Failed to create index");

    // Query information_schema.statistics
    let result = db
        .query(
            "SELECT index_name, column_name, non_unique
             FROM \"information_schema\".\"statistics\"
             WHERE table_name = 'users'
             ORDER BY index_name, seq_in_index",
            (),
        )
        .expect("Failed to query information_schema.statistics");

    let mut indexes = Vec::new();
    for row in result {
        let row = row.expect("Failed to read row");
        let index_name: String = row.get(0).expect("Failed to get index_name");
        let column_name: String = row.get(1).expect("Failed to get column_name");
        let non_unique: bool = row.get(2).expect("Failed to get non_unique");
        indexes.push((index_name, column_name, non_unique));
    }

    assert_eq!(indexes.len(), 2);
    assert_eq!(
        indexes[0],
        ("idx_email".to_string(), "email".to_string(), true)
    );
    assert_eq!(
        indexes[1],
        ("idx_name".to_string(), "name".to_string(), true)
    );
}

#[test]
fn test_show_functions() {
    let db = Database::open("memory://show_functions").expect("Failed to create database");

    // Test SHOW FUNCTION
    let result = db
        .query("SHOW FUNCTION", ())
        .expect("Failed to execute SHOW FUNCTION");

    let mut functions = Vec::new();
    for row in result {
        let row = row.expect("Failed to read row");
        let name: String = row.get(0).expect("Failed to get name");
        let func_type: String = row.get(1).expect("Failed to get type");
        functions.push((name, func_type));
    }

    // Should have many functions
    assert!(functions.len() > 50);

    // Check that we have different types
    let has_scalar = functions.iter().any(|(_, t)| t == "SCALAR");
    let has_aggregate = functions.iter().any(|(_, t)| t == "AGGREGATE");
    let has_window = functions.iter().any(|(_, t)| t == "WINDOW");

    assert!(has_scalar);
    assert!(has_aggregate);
    assert!(has_window);
}

#[test]
fn test_information_schema_basic_queries() {
    let db = Database::open("memory://basic_queries").expect("Failed to create database");

    // Create some test data
    db.execute("CREATE TABLE users (id INTEGER, name TEXT)", ())
        .expect("Failed to create table");
    db.execute(
        "CREATE VIEW active_users AS SELECT * FROM users WHERE id > 0",
        (),
    )
    .expect("Failed to create view");

    // Test SELECT * FROM information_schema.tables (with quotes)
    let result = db
        .query("SELECT * FROM \"information_schema\".\"tables\"", ())
        .expect("Failed to query information_schema.tables");

    let rows: Vec<_> = result.collect();
    assert_eq!(rows.len(), 2); // users table and active_users view

    // Test SHOW FUNCTION
    let result = db
        .query("SHOW FUNCTION", ())
        .expect("Failed to execute SHOW FUNCTION");

    let rows: Vec<_> = result.collect();
    assert!(rows.len() > 10); // Should have many functions
}

#[test]
fn test_information_schema_table_schema_multiple_schemas() {
    let db =
        Database::open("memory://info_schema_multi_schema").expect("Failed to create database");

    // Create tables in default schema
    db.execute("CREATE TABLE default_table (id INTEGER, name TEXT)", ())
        .expect("Failed to create table in default schema");

    // Create a new schema and table
    db.execute("CREATE SCHEMA test_schema", ())
        .expect("Failed to create schema");
    db.execute(
        "CREATE TABLE test_schema.test_table (id INTEGER, value FLOAT)",
        (),
    )
    .expect("Failed to create table in test schema");

    // Query information_schema.tables
    let result = db
        .query(
            "SELECT table_schema, table_name, table_type FROM information_schema.tables ORDER BY table_schema, table_name",
            (),
        )
        .expect("Failed to query information_schema.tables");

    let mut tables = Vec::new();
    for row in result {
        let row = row.expect("Failed to read row");
        let table_schema: Option<String> = row.get(0).ok();
        let table_name: String = row.get(1).expect("Failed to get table_name");
        let table_type: String = row.get(2).expect("Failed to get table_type");
        tables.push((table_schema, table_name, table_type));
    }

    // Should have 2 tables from different schemas
    assert_eq!(tables.len(), 2);

    // Tables should be sorted by schema, then table_name
    // Check that we have the expected tables
    let public_table = tables
        .iter()
        .find(|(schema, name, _)| schema.as_ref().unwrap() == "public" && name == "default_table")
        .expect("Should find default_table in public schema");

    assert_eq!(public_table.2, "BASE TABLE");

    let test_schema_table = tables
        .iter()
        .find(|(schema, name, _)| schema.as_ref().unwrap() == "test_schema" && name == "test_table")
        .expect("Should find test_table in test_schema");

    assert_eq!(test_schema_table.2, "BASE TABLE");
}
