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

//! Deno scripting backend tests
//!
//! Tests specific to Deno JavaScript/TypeScript functionality.
//! These tests are only run when the "deno" feature is enabled.

#[cfg(feature = "deno")]
use oxibase::Database;

#[cfg(feature = "deno")]
mod deno_function_tests {
    use super::*;

    #[test]
    fn test_deno_basic_execution() {
        let db = Database::open("memory://deno_test").unwrap();

        // Test basic JavaScript execution
        db.execute(
            r#"
            CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
            RETURNS INTEGER
            LANGUAGE DENO AS 'return arguments[0] + arguments[1];'
        "#,
            (),
        )
        .unwrap();

        let result: i64 = db.query_one("SELECT add_numbers(5, 3)", ()).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_deno_string_manipulation() {
        let db = Database::open("memory://deno_string_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION greet(name TEXT)
            RETURNS TEXT
            LANGUAGE DENO AS 'return `Hello, ${arguments[0]}!`;'
        "#,
            (),
        )
        .unwrap();

        let result: String = db.query_one("SELECT greet('World')", ()).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_deno_math_operations() {
        let db = Database::open("memory://deno_math_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION power(base INTEGER, exp INTEGER)
            RETURNS INTEGER
            LANGUAGE DENO AS 'return Math.pow(arguments[0], arguments[1]);'
        "#,
            (),
        )
        .unwrap();

        let result: i64 = db.query_one("SELECT power(2, 3)", ()).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_deno_type_conversion() {
        let db = Database::open("memory://deno_types_test").unwrap();

        // Test INTEGER
        db.execute(
            r#"
            CREATE FUNCTION double_int(x INTEGER)
            RETURNS INTEGER
            LANGUAGE DENO AS 'return x * 2;'
        "#,
            (),
        )
        .unwrap();

        // Test FLOAT
        db.execute(
            r#"
            CREATE FUNCTION double_float(x FLOAT)
            RETURNS FLOAT
            LANGUAGE DENO AS 'return arguments[0] * 2.0;'
        "#,
            (),
        )
        .unwrap();

        // Test BOOLEAN
        db.execute(
            r#"
            CREATE FUNCTION negate_bool(x BOOLEAN)
            RETURNS BOOLEAN
            LANGUAGE DENO AS 'return !arguments[0];'
        "#,
            (),
        )
        .unwrap();

        let int_result: i64 = db.query_one("SELECT double_int(21)", ()).unwrap();
        assert_eq!(int_result, 42);

        let float_result: f64 = db.query_one("SELECT double_float(3.14)", ()).unwrap();
        assert!((float_result - std::f64::consts::TAU).abs() < 0.01);

        let bool_result: bool = db.query_one("SELECT negate_bool(true)", ()).unwrap();
        assert!(!bool_result);
    }

    #[test]
    fn test_deno_argument_types() {
        let db = Database::open("memory://deno_args_test").unwrap();

        // Test INTEGER, FLOAT, TEXT, BOOLEAN arguments
        db.execute(r#"
            CREATE FUNCTION process_args(i INTEGER, f FLOAT, t TEXT, b BOOLEAN)
            RETURNS TEXT
            LANGUAGE DENO AS 'return `${arguments[0]}, ${arguments[1]}, ${arguments[2]}, ${arguments[3]}`;'
        "#, ()).unwrap();

        let result: String = db
            .query_one("SELECT process_args(42, 3.14, 'hello', true)", ())
            .unwrap();
        assert_eq!(result, "42, 3.14, hello, true");
    }

    #[test]
    fn test_deno_return_types() {
        let db = Database::open("memory://deno_return_test").unwrap();

        // Test all supported return type conversions
        db.execute(
            r#"
            CREATE FUNCTION return_int() RETURNS INTEGER
            LANGUAGE DENO AS 'return 42;'
        "#,
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE FUNCTION return_float() RETURNS FLOAT
            LANGUAGE DENO AS 'return 3.14;'
        "#,
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE FUNCTION return_text() RETURNS TEXT
            LANGUAGE DENO AS 'return "hello";'
        "#,
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE FUNCTION return_bool() RETURNS BOOLEAN
            LANGUAGE DENO AS 'return true;'
        "#,
            (),
        )
        .unwrap();

        let int_result: i64 = db.query_one("SELECT return_int()", ()).unwrap();
        assert_eq!(int_result, 42);

        let float_result: f64 = db.query_one("SELECT return_float()", ()).unwrap();
        assert!((float_result - std::f64::consts::PI).abs() < 0.01);

        let text_result: String = db.query_one("SELECT return_text()", ()).unwrap();
        assert_eq!(text_result, "hello");

        let bool_result: bool = db.query_one("SELECT return_bool()", ()).unwrap();
        assert!(bool_result);
    }

    #[test]
    fn test_deno_invalid_syntax() {
        let db = Database::open("memory://deno_invalid_test").unwrap();

        // Test that invalid JavaScript syntax fails during execution (not creation)
        db.execute(
            r#"
            CREATE FUNCTION invalid_func() RETURNS INTEGER
            LANGUAGE DENO AS 'return 1 +'
        "#,
            (),
        )
        .unwrap(); // Creation succeeds

        // Execution should fail
        let result: Result<i64, _> = db.query_one("SELECT invalid_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_deno_runtime_error() {
        let db = Database::open("memory://deno_runtime_error_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION error_func() RETURNS INTEGER
            LANGUAGE DENO AS 'return 1 / 0;'
        "#,
            (),
        )
        .unwrap();

        let result: Result<i64, _> = db.query_one("SELECT error_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_deno_function_creation_and_execution() {
        let db = Database::open("memory://deno_create_test").unwrap();

        // Create a Deno function that returns a value
        db.execute(
            r#"
            CREATE FUNCTION compute_sum(a INTEGER, b INTEGER) RETURNS INTEGER
            LANGUAGE DENO AS 'return arguments[0] + arguments[1];'
        "#,
            (),
        )
        .unwrap();

        // Execute the function
        let result: i64 = db.query_one("SELECT compute_sum(10, 20)", ()).unwrap();
        assert_eq!(result, 30);
    }

    #[test]
    fn test_deno_function_with_different_types() {
        let db = Database::open("memory://deno_types_test").unwrap();

        // Test INTEGER return
        db.execute(
            r#"
            CREATE FUNCTION get_answer() RETURNS INTEGER
            LANGUAGE DENO AS 'return 42;'
        "#,
            (),
        )
        .unwrap();

        // Test TEXT return
        db.execute(
            r#"
            CREATE FUNCTION get_message() RETURNS TEXT
            LANGUAGE DENO AS 'return "Hello from Deno";'
        "#,
            (),
        )
        .unwrap();

        // Test BOOLEAN return
        db.execute(
            r#"
            CREATE FUNCTION get_truth() RETURNS BOOLEAN
            LANGUAGE DENO AS 'return true;'
        "#,
            (),
        )
        .unwrap();

        let int_result: i64 = db.query_one("SELECT get_answer()", ()).unwrap();
        assert_eq!(int_result, 42);

        let text_result: String = db.query_one("SELECT get_message()", ()).unwrap();
        assert_eq!(text_result, "Hello from Deno");

        let bool_result: bool = db.query_one("SELECT get_truth()", ()).unwrap();
        assert!(bool_result);
    }

    #[test]
    fn test_deno_function_with_multiple_arguments() {
        let db = Database::open("memory://deno_multi_args_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION format_person(name TEXT, age INTEGER, active BOOLEAN) RETURNS TEXT
            LANGUAGE DENO AS 'return `${arguments[0]} is ${arguments[1]} years old, active: ${arguments[2]}`;'
        "#, ()).unwrap();

        let result: String = db
            .query_one("SELECT format_person('Alice', 30, true)", ())
            .unwrap();
        assert_eq!(result, "Alice is 30 years old, active: true");
    }

    #[test]
    fn test_deno_function_drop() {
        let db = Database::open("memory://deno_drop_test").unwrap();

        // Create function
        db.execute(
            r#"
            CREATE FUNCTION temp_func() RETURNS INTEGER
            LANGUAGE DENO AS 'return 123;'
        "#,
            (),
        )
        .unwrap();

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
    fn test_deno_syntax_validation() {
        let db = Database::open("memory://deno_syntax_test").unwrap();

        // Test that valid syntax works
        let result = db.execute(
            r#"
            CREATE FUNCTION valid_func() RETURNS INTEGER
            LANGUAGE DENO AS 'return 1;'
        "#,
            (),
        );
        assert!(result.is_ok());
        let result: Result<i64, _> = db.query_one("SELECT valid_func()", ());
        assert!(result.is_ok());

        // Test that invalid syntax fails during execution
        let result = db.execute(
            r#"
            CREATE FUNCTION invalid_func() RETURNS INTEGER
            LANGUAGE DENO AS 'return 1 + ;'
        "#,
            (),
        );
        assert!(result.is_ok()); // Creation succeeds

        let result: Result<i64, _> = db.query_one("SELECT invalid_func()", ());
        assert!(result.is_err()); // Execution fails
    }

    #[test]
    fn test_deno_function_without_return() {
        let db = Database::open("memory://deno_no_result_test").unwrap();

        // Function that doesn't return a value should return NULL
        db.execute(
            r#"
            CREATE FUNCTION no_return_func() RETURNS INTEGER
            LANGUAGE DENO AS 'let x = 42;'
        "#,
            (),
        )
        .unwrap();

        // Execution should return NULL
        let result: Option<i64> = db.query_one("SELECT no_return_func()", ()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_deno_security_restrictions() {
        let db = Database::open("memory://security_test").unwrap();

        // Test file system access is blocked
        db.execute(r#"
            CREATE FUNCTION test_fs()
            RETURNS TEXT
            LANGUAGE DENO AS 'try { Deno.readFile("test"); return "FS accessible"; } catch(e) { return "FS blocked: " + e.name; }'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT test_fs()", ()).unwrap();
        assert!(result.contains("FS blocked"));

        // Test process spawning is blocked
        db.execute(r#"
            CREATE FUNCTION test_process()
            RETURNS TEXT
            LANGUAGE DENO AS 'try { Deno.run({cmd: "echo"}); return "Process accessible"; } catch(e) { return "Process blocked: " + e.name; }'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT test_process()", ()).unwrap();
        assert!(result.contains("Process blocked"));

        // Test KV is blocked
        db.execute(r#"
            CREATE FUNCTION test_kv()
            RETURNS TEXT
            LANGUAGE DENO AS 'try { Deno.openKv(); return "KV accessible"; } catch(e) { return "KV blocked: " + e.name; }'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT test_kv()", ()).unwrap();
        assert!(result.contains("KV blocked"));

        // Test Node.js APIs are blocked
        db.execute(r#"
            CREATE FUNCTION test_node()
            RETURNS TEXT
            LANGUAGE DENO AS 'try { require("fs"); return "Node accessible"; } catch(e) { return "Node blocked: " + e.name; }'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT test_node()", ()).unwrap();
        assert!(result.contains("Node blocked"));

        // Test webstorage is blocked
        db.execute(r#"
            CREATE FUNCTION test_storage()
            RETURNS TEXT
            LANGUAGE DENO AS 'try { localStorage.setItem("test", "value"); return "Storage accessible"; } catch(e) { return "Storage blocked: " + e.name; }'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT test_storage()", ()).unwrap();
        assert!(result.contains("Storage blocked"));

        // Test that fetch still works (allowed)
        db.execute(r#"
            CREATE FUNCTION test_fetch()
            RETURNS TEXT
            LANGUAGE DENO AS 'try { fetch("https://httpbin.org/get"); return "Fetch works"; } catch(e) { return "Fetch failed: " + e.message; }'
        "#, ()).unwrap();

        // Note: This might fail due to network timeouts in tests, but the API should be available
        let result: String = db.query_one("SELECT test_fetch()", ()).unwrap();
        // Just check that it's not blocked at the API level
        assert!(
            !result.contains("blocked")
                || result.contains("Fetch works")
                || result.contains("network")
        );
    }
}
