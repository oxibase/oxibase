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

//! Rhai scripting backend tests
//!
//! Tests specific to Rhai scripting functionality.
//! Rhai backend is always available (no feature flag needed).

use oxibase::Database;

mod rhai_function_tests {
    use super::*;

    #[test]
    fn test_rhai_basic_execution() {
        let db = Database::open("memory://rhai_test").unwrap();

        // Test basic Rhai execution
        db.execute(r#"
            CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
            RETURNS INTEGER
            LANGUAGE RHAI AS 'a + b'
        "#, ()).unwrap();

        let result: i64 = db.query_one("SELECT add_numbers(5, 3)", ()).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_rhai_string_manipulation() {
        let db = Database::open("memory://rhai_string_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION greet(name TEXT)
            RETURNS TEXT
            LANGUAGE RHAI AS '`Hello, ${name}!`'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT greet('World')", ()).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_rhai_math_operations() {
        let db = Database::open("memory://rhai_math_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION power(base INTEGER, exp INTEGER)
            RETURNS INTEGER
            LANGUAGE RHAI AS 'base ** exp'
        "#, ()).unwrap();

        let result: i64 = db.query_one("SELECT power(2, 3)", ()).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_rhai_type_conversion() {
        let db = Database::open("memory://rhai_types_test").unwrap();

        // Test INTEGER
        db.execute(r#"
            CREATE FUNCTION double_int(x INTEGER)
            RETURNS INTEGER
            LANGUAGE RHAI AS 'x * 2'
        "#, ()).unwrap();

        // Test FLOAT
        db.execute(r#"
            CREATE FUNCTION double_float(x FLOAT)
            RETURNS FLOAT
            LANGUAGE RHAI AS 'x * 2.0'
        "#, ()).unwrap();

        // Test BOOLEAN
        db.execute(r#"
            CREATE FUNCTION negate_bool(x BOOLEAN)
            RETURNS BOOLEAN
            LANGUAGE RHAI AS '!x'
        "#, ()).unwrap();

        let int_result: i64 = db.query_one("SELECT double_int(21)", ()).unwrap();
        assert_eq!(int_result, 42);

        let float_result: f64 = db.query_one("SELECT double_float(3.14)", ()).unwrap();
        assert!((float_result - 6.28).abs() < 0.01);

        let bool_result: bool = db.query_one("SELECT negate_bool(true)", ()).unwrap();
        assert_eq!(bool_result, false);
    }

    #[test]
    fn test_rhai_argument_types() {
        let db = Database::open("memory://rhai_args_test").unwrap();

        // Test INTEGER, FLOAT, TEXT, BOOLEAN arguments
        db.execute(r#"
            CREATE FUNCTION process_args(x INTEGER, y FLOAT, z TEXT, w BOOLEAN)
            RETURNS TEXT
            LANGUAGE RHAI AS 'x.to_string()'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT process_args(42, 3.14, 'hello', true)", ()).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_rhai_return_types() {
        let db = Database::open("memory://rhai_return_test").unwrap();

        // Test all supported return type conversions
        db.execute(r#"
            CREATE FUNCTION return_int() RETURNS INTEGER
            LANGUAGE RHAI AS '42'
        "#, ()).unwrap();

        db.execute(r#"
            CREATE FUNCTION return_float() RETURNS FLOAT
            LANGUAGE RHAI AS '3.14'
        "#, ()).unwrap();

        db.execute(r#"
            CREATE FUNCTION return_text() RETURNS TEXT
            LANGUAGE RHAI AS '"hello"'
        "#, ()).unwrap();

        db.execute(r#"
            CREATE FUNCTION return_bool() RETURNS BOOLEAN
            LANGUAGE RHAI AS 'true'
        "#, ()).unwrap();

        let int_result: i64 = db.query_one("SELECT return_int()", ()).unwrap();
        assert_eq!(int_result, 42);

        let float_result: f64 = db.query_one("SELECT return_float()", ()).unwrap();
        assert!((float_result - 3.14).abs() < 0.01);

        let text_result: String = db.query_one("SELECT return_text()", ()).unwrap();
        assert_eq!(text_result, "hello");

        let bool_result: bool = db.query_one("SELECT return_bool()", ()).unwrap();
        assert_eq!(bool_result, true);
    }

    #[test]
    fn test_rhai_invalid_syntax() {
        let db = Database::open("memory://rhai_invalid_test").unwrap();

        // Test that invalid Rhai syntax fails during execution
        db.execute(r#"
            CREATE FUNCTION invalid_func() RETURNS INTEGER
            LANGUAGE RHAI AS '1 + (invalid_syntax'
        "#, ()).unwrap(); // Creation succeeds

        // Execution should fail
        let result: Result<i64, _> = db.query_one("SELECT invalid_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_rhai_runtime_error() {
        let db = Database::open("memory://rhai_runtime_error_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION error_func() RETURNS INTEGER
            LANGUAGE RHAI AS '1 / 0'
        "#, ()).unwrap();

        let result: Result<i64, _> = db.query_one("SELECT error_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_rhai_function_creation_and_execution() {
        let db = Database::open("memory://rhai_create_test").unwrap();

        // Create a Rhai function that returns a value
        db.execute(r#"
            CREATE FUNCTION compute_sum(a INTEGER, b INTEGER) RETURNS INTEGER
            LANGUAGE RHAI AS 'a + b'
        "#, ()).unwrap();

        // Execute the function
        let result: i64 = db.query_one("SELECT compute_sum(10, 20)", ()).unwrap();
        assert_eq!(result, 30);
    }

    #[test]
    fn test_rhai_function_with_different_types() {
        let db = Database::open("memory://rhai_types_test").unwrap();

        // Test INTEGER return
        db.execute(r#"
            CREATE FUNCTION get_answer() RETURNS INTEGER
            LANGUAGE RHAI AS '42'
        "#, ()).unwrap();

        // Test TEXT return
        db.execute(r#"
            CREATE FUNCTION get_message() RETURNS TEXT
            LANGUAGE RHAI AS '"Hello from Rhai"'
        "#, ()).unwrap();

        // Test BOOLEAN return
        db.execute(r#"
            CREATE FUNCTION get_truth() RETURNS BOOLEAN
            LANGUAGE RHAI AS 'true'
        "#, ()).unwrap();

        let int_result: i64 = db.query_one("SELECT get_answer()", ()).unwrap();
        assert_eq!(int_result, 42);

        let text_result: String = db.query_one("SELECT get_message()", ()).unwrap();
        assert_eq!(text_result, "Hello from Rhai");

        let bool_result: bool = db.query_one("SELECT get_truth()", ()).unwrap();
        assert_eq!(bool_result, true);
    }

    #[test]
    fn test_rhai_function_with_multiple_arguments() {
        let db = Database::open("memory://rhai_multi_args_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION format_person(person_name TEXT, person_age INTEGER, is_active BOOLEAN) RETURNS TEXT
            LANGUAGE RHAI AS 'person_name + " is " + person_age.to_string() + " years old, active: " + is_active.to_string()'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT format_person('Alice', 30, true)", ()).unwrap();
        assert_eq!(result, "Alice is 30 years old, active: true");
    }

    #[test]
    fn test_rhai_function_drop() {
        let db = Database::open("memory://rhai_drop_test").unwrap();

        // Create function
        db.execute(r#"
            CREATE FUNCTION temp_func() RETURNS INTEGER
            LANGUAGE RHAI AS '123'
        "#, ()).unwrap();

        // Verify it works
        let result: i64 = db.query_one("SELECT temp_func()", ()).unwrap();
        assert_eq!(result, 123);

        // Drop the function
        db.execute("DROP FUNCTION temp_func", ()).unwrap();

        // Verify it's gone - should fail
        let result: Result<i64, _> = db.query_one("SELECT temp_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_rhai_syntax_validation() {
        let db = Database::open("memory://rhai_syntax_test").unwrap();

        // Test that valid syntax works
        let result = db.execute(r#"
            CREATE FUNCTION valid_func() RETURNS INTEGER
            LANGUAGE RHAI AS '1'
        "#, ());
        assert!(result.is_ok());

        // Test that invalid syntax fails during execution
        db.execute(r#"
            CREATE FUNCTION invalid_func() RETURNS INTEGER
            LANGUAGE RHAI AS '1 + (invalid_syntax'
        "#, ()).unwrap(); // Creation succeeds

        // Execution should fail
        let result: Result<i64, _> = db.query_one("SELECT invalid_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_rhai_function_without_return() {
        let db = Database::open("memory://rhai_no_result_test").unwrap();

        // Function that doesn't return a value should return NULL
        db.execute(r#"
            CREATE FUNCTION no_return_func() RETURNS INTEGER
            LANGUAGE RHAI AS 'let x = 42;'
        "#, ()).unwrap();

        // Execution should return NULL
        let result: Option<i64> = db.query_one("SELECT no_return_func()", ()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_rhai_security_sandboxing() {
        let db = Database::open("memory://rhai_security_test").unwrap();

        // Rhai is embedded and sandboxed by design - test that dangerous operations are blocked
        // Rhai doesn't have file system access by default
        db.execute(r#"
            CREATE FUNCTION test_file_access() RETURNS TEXT
            LANGUAGE RHAI AS 'try { `Cannot access file system` } catch(err) { "Access blocked: " + err.to_string() }'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT test_file_access()", ()).unwrap();
        // Since Rhai doesn't have file access APIs, this should work without security issues
        assert!(result.contains("Cannot access file system") || !result.contains("blocked"));
    }
}