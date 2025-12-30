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

//! Python scripting backend tests
//!
//! Tests specific to Python functionality.
//! These tests are only run when the "python" feature is enabled.

#[cfg(feature = "python")]
use oxibase::Database;

#[cfg(feature = "python")]
mod python_function_tests {
    use super::*;

    #[test]
    fn test_python_function_creation_and_execution() {
        let db = Database::open("memory://python_create_test").unwrap();

        // Create a Python function that sets result variable
        db.execute(r#"
            CREATE FUNCTION compute_sum(a INTEGER, b INTEGER) RETURNS INTEGER
            LANGUAGE PYTHON AS 'result = arguments[0] + arguments[1]'
        "#, ()).unwrap();

        // Execute the function
        let result: i64 = db.query_one("SELECT compute_sum(10, 20)", ()).unwrap();
        assert_eq!(result, 30);
    }

    #[test]
    fn test_python_function_with_different_types() {
        let db = Database::open("memory://python_types_test").unwrap();

        // Test INTEGER return
        db.execute(r#"
            CREATE FUNCTION get_answer() RETURNS INTEGER
            LANGUAGE PYTHON AS 'result = 42'
        "#, ()).unwrap();

        // Test TEXT return
        db.execute(r#"
            CREATE FUNCTION get_message() RETURNS TEXT
            LANGUAGE PYTHON AS 'result = "Hello from Python"'
        "#, ()).unwrap();

        // Test BOOLEAN return
        db.execute(r#"
            CREATE FUNCTION get_truth() RETURNS BOOLEAN
            LANGUAGE PYTHON AS 'result = True'
        "#, ()).unwrap();

        let int_result: i64 = db.query_one("SELECT get_answer()", ()).unwrap();
        assert_eq!(int_result, 42);

        let text_result: String = db.query_one("SELECT get_message()", ()).unwrap();
        assert_eq!(text_result, "Hello from Python");

        let bool_result: bool = db.query_one("SELECT get_truth()", ()).unwrap();
        assert_eq!(bool_result, true);
    }

    #[test]
    fn test_python_function_with_multiple_arguments() {
        let db = Database::open("memory://python_multi_args_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION format_person(name TEXT, age INTEGER, active BOOLEAN) RETURNS TEXT
            LANGUAGE PYTHON AS 'result = f"{arguments[0]} is {arguments[1]} years old, active: {arguments[2]}"'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT format_person('Alice', 30, true)", ()).unwrap();
        assert_eq!(result, "Alice is 30 years old, active: True");
    }

    #[test]
    fn test_python_function_drop() {
        let db = Database::open("memory://python_drop_test").unwrap();

        // Create function
        db.execute(r#"
            CREATE FUNCTION temp_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'result = 123'
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
    fn test_python_syntax_validation() {
        let db = Database::open("memory://python_syntax_test").unwrap();

        // Test that valid syntax works
        let result = db.execute(r#"
            CREATE FUNCTION valid_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'result = 1'
        "#, ());
        assert!(result.is_ok());

        // Test that invalid syntax fails
        let result = db.execute(r#"
            CREATE FUNCTION invalid_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'result = 1 +'
        "#, ());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("syntax error"));
    }

    #[test]
    fn test_python_function_without_result() {
        let db = Database::open("memory://python_no_result_test").unwrap();

        // Function that doesn't set result should fail
        let result = db.execute(r#"
            CREATE FUNCTION no_result_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'x = 42'
        "#, ());
        assert!(result.is_ok()); // Creation succeeds, but execution fails

        // Execution should fail
        let result: Result<i64, _> = db.query_one("SELECT no_result_func()", ());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must set a 'result' variable"));
    }

    #[test]
    fn test_python_runtime_error() {
        let db = Database::open("memory://python_runtime_error_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION error_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'result = 1 / 0'
        "#, ()).unwrap();

        let result: Result<i64, _> = db.query_one("SELECT error_func()", ());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("execution error"));
        // Should include Python exception details
    }
}

    #[test]
    fn test_python_function_creation_fails() {
        let db = Database::open("memory://python_func_test").unwrap();

        // Try to call a Python function - should fail
        let result: Result<bool, _> = db.query_one("SELECT is_even(4)", ());

        // Should fail since function doesn't exist or backend not implemented
        assert!(result.is_err());
    }

    // TODO: Add these tests once Python backend is fully implemented:
    /*
    #[test]
    fn test_python_basic_execution() {
        let db = Database::open("memory://python_exec_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION is_even(n INTEGER)
            RETURNS BOOLEAN
            LANGUAGE PYTHON AS 'return arguments[0] % 2 == 0'
        "#, ()).unwrap();

        let result: bool = db.query_one("SELECT is_even(4)", ()).unwrap();
        assert_eq!(result, true);

        let result: bool = db.query_one("SELECT is_even(5)", ()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_python_argument_types() {
        let db = Database::open("memory://python_args_test").unwrap();

        // Test INTEGER, FLOAT, TEXT, BOOLEAN arguments
        db.execute(r#"
            CREATE FUNCTION process_args(i INTEGER, f FLOAT, t TEXT, b BOOLEAN)
            RETURNS TEXT
            LANGUAGE PYTHON AS 'return f"{arguments[0]}, {arguments[1]}, {arguments[2]}, {arguments[3]}"'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT process_args(42, 3.14, 'hello', true)", ()).unwrap();
        assert_eq!(result, "42, 3.14, hello, True");
    }

    #[test]
    fn test_python_return_types() {
        let db = Database::open("memory://python_return_test").unwrap();

        // Test all supported return type conversions
        db.execute(r#"
            CREATE FUNCTION return_int() RETURNS INTEGER
            LANGUAGE PYTHON AS 'return 42'
        "#, ()).unwrap();

        db.execute(r#"
            CREATE FUNCTION return_float() RETURNS FLOAT
            LANGUAGE PYTHON AS 'return 3.14'
        "#, ()).unwrap();

        db.execute(r#"
            CREATE FUNCTION return_text() RETURNS TEXT
            LANGUAGE PYTHON AS 'return "hello"'
        "#, ()).unwrap();

        db.execute(r#"
            CREATE FUNCTION return_bool() RETURNS BOOLEAN
            LANGUAGE PYTHON AS 'return True'
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
    */
}