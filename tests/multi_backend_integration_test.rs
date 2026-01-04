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

//! Multi-backend integration tests
//!
//! Tests for user-defined functions from different backends working together
//! and cross-language function interactions.

use oxibase::Database;

#[cfg(test)]
mod multi_backend_integration_tests {
    use super::*;

    #[test]
    fn test_multi_backend_function_creation() {
        let db = Database::open("memory://multi_backend_test").unwrap();

        // Create functions in different backends
        db.execute(
            r#"
            CREATE FUNCTION rhai_add(a INTEGER, b INTEGER) RETURNS INTEGER
            LANGUAGE RHAI AS 'a + b'
        "#,
            (),
        )
        .unwrap();

        #[cfg(feature = "js")]
        db.execute(
            r#"
            CREATE FUNCTION deno_multiply(a INTEGER, b INTEGER) RETURNS INTEGER
            LANGUAGE DENO AS 'return a * b;'
        "#,
            (),
        )
        .unwrap();

        #[cfg(feature = "python")]
        db.execute(
            r#"
            CREATE FUNCTION python_subtract(a INTEGER, b INTEGER) RETURNS INTEGER
            LANGUAGE PYTHON AS 'return a - b'
        "#,
            (),
        )
        .unwrap();

        // Test Rhai function
        let rhai_result: Result<i64, _> = db.query_one("SELECT rhai_add(10, 5)", ());
        assert!(
            rhai_result.is_ok(),
            "Rhai function should execute successfully"
        );
        assert_eq!(rhai_result.unwrap(), 15);

        #[cfg(feature = "js")]
        {
            let deno_result: Result<i64, _> = db.query_one("SELECT deno_multiply(4, 3)", ());
            assert!(
                deno_result.is_ok(),
                "Deno function should execute successfully"
            );
            assert_eq!(deno_result.unwrap(), 12);
        }

        #[cfg(feature = "python")]
        {
            let python_result: Result<i64, _> = db.query_one("SELECT python_subtract(20, 7)", ());
            assert!(
                python_result.is_ok(),
                "Python function should execute successfully"
            );
            assert_eq!(python_result.unwrap(), 13);
        }
    }

    #[test]
    fn test_cross_language_function_calls() {
        let db = Database::open("memory://cross_lang_test").unwrap();

        // Create helper functions in different languages
        db.execute(
            r#"
            CREATE FUNCTION get_base() RETURNS INTEGER
            LANGUAGE RHAI AS '10'
        "#,
            (),
        )
        .unwrap();

        #[cfg(feature = "js")]
        db.execute(
            r#"
            CREATE FUNCTION double_value(x INTEGER) RETURNS INTEGER
            LANGUAGE DENO AS 'return x * 2;'
        "#,
            (),
        )
        .unwrap();

        // Test calling Rhai from SQL
        let base: Result<i64, _> = db.query_one("SELECT get_base()", ());
        assert!(base.is_ok(), "get_base should work");
        assert_eq!(base.unwrap(), 10);

        #[cfg(feature = "js")]
        {
            // Test calling Deno from SQL
            let doubled: Result<i64, _> = db.query_one("SELECT double_value(5)", ());
            assert!(doubled.is_ok(), "double_value should work");
            assert_eq!(doubled.unwrap(), 10);
        }

        // Test composing functions in a single query
        #[cfg(feature = "js")]
        {
            let composed: Result<i64, _> = db.query_one("SELECT double_value(get_base())", ());
            assert!(composed.is_ok(), "Composed functions should work");
            assert_eq!(composed.unwrap(), 20);
        }
    }

    #[test]
    fn test_backend_registry_integration() {
        let db = Database::open("memory://registry_integration_test").unwrap();

        // Test that functions from all available backends can be created and called
        #[allow(clippy::useless_vec)]
        let backends = vec![
            ("RHAI", "42"),
            #[cfg(feature = "js")]
            ("DENO", "return 42;"),
            #[cfg(feature = "python")]
            ("PYTHON", "return 42"),
        ];

        for (i, (lang, code)) in backends.iter().enumerate() {
            let func_name = format!("test_func_{}", i);
            let sql = format!(
                r#"
                CREATE FUNCTION {}() RETURNS INTEGER
                LANGUAGE {} AS '{}'
            "#,
                func_name, lang, code
            );

            db.execute(&sql, ()).unwrap();

            let result: Result<i64, _> = db.query_one(&format!("SELECT {}()", func_name), ());
            assert!(result.is_ok(), "Function {} should execute", func_name);
            assert_eq!(result.unwrap(), 42);
        }
    }

    #[test]
    fn test_function_overloading_different_languages() {
        // Note: This tests if different languages can have functions with same name
        // Currently, function names must be unique across all backends
        let db = Database::open("memory://overloading_test").unwrap();

        // Create function in Rhai
        db.execute(
            r#"
            CREATE FUNCTION shared_name() RETURNS TEXT
            LANGUAGE RHAI AS '"rhai_result"'
        "#,
            (),
        )
        .unwrap();

        // Try to create function with same name in different language - should fail
        #[cfg(feature = "js")]
        {
            let result = db.execute(
                r#"
                CREATE FUNCTION shared_name() RETURNS TEXT
                LANGUAGE DENO AS 'return "deno_result";'
            "#,
                (),
            );
            assert!(
                result.is_err(),
                "Should not allow duplicate function names across backends"
            );
        }

        // Verify original function still works
        let result: Result<String, _> = db.query_one("SELECT shared_name()", ());
        assert!(result.is_ok(), "Original function should still work");
        assert_eq!(result.unwrap(), "rhai_result");
    }

    #[test]
    fn test_show_functions_multi_backend() {
        let db = Database::open("memory://show_functions_test").unwrap();

        // Create functions in different backends
        db.execute(
            r#"
            CREATE FUNCTION func_rhai() RETURNS INTEGER
            LANGUAGE RHAI AS '1'
        "#,
            (),
        )
        .unwrap();

        #[cfg(feature = "js")]
        db.execute(
            r#"
            CREATE FUNCTION func_deno() RETURNS INTEGER
            LANGUAGE DENO AS 'return 2;'
        "#,
            (),
        )
        .unwrap();

        #[cfg(feature = "python")]
        db.execute(
            r#"
            CREATE FUNCTION func_python() RETURNS INTEGER
            LANGUAGE PYTHON AS 'return 3'
        "#,
            (),
        )
        .unwrap();

        // Check SHOW FUNCTIONS lists all
        let result = db.query("SHOW FUNCTIONS", ()).unwrap();
        let functions: Vec<String> = result
            .map(|row| row.unwrap().get::<String>(0).unwrap())
            .collect();

        assert!(functions.contains(&"FUNC_RHAI".to_string()));

        #[cfg(feature = "js")]
        assert!(functions.contains(&"FUNC_DENO".to_string()));

        #[cfg(feature = "python")]
        assert!(functions.contains(&"FUNC_PYTHON".to_string()));
    }

    #[test]
    fn test_information_schema_functions_multi_backend() {
        let db = Database::open("memory://info_schema_test").unwrap();

        // Create functions in different backends
        db.execute(
            r#"
            CREATE FUNCTION info_rhai() RETURNS TEXT
            LANGUAGE RHAI AS '"rhai"'
        "#,
            (),
        )
        .unwrap();

        #[cfg(feature = "js")]
        db.execute(
            r#"
            CREATE FUNCTION info_deno() RETURNS TEXT
            LANGUAGE DENO AS 'return "deno";'
        "#,
            (),
        )
        .unwrap();

        // Query information_schema.functions - note: language is not exposed in schema
        let result = db
            .query(
                r#"
            SELECT function_name, function_type, data_type
            FROM information_schema.functions
            WHERE function_name LIKE 'INFO_%'
            ORDER BY function_name
        "#,
                (),
            )
            .unwrap();

        let rows: Vec<_> = result.collect();

        // Should have at least one function
        assert!(!rows.is_empty());

        // Check that our functions are listed
        let function_names: Vec<String> = rows
            .iter()
            .filter_map(|row| row.as_ref().ok().map(|r| r.get::<String>(0).unwrap()))
            .collect();

        assert!(function_names.contains(&"INFO_RHAI".to_string()));

        #[cfg(feature = "js")]
        assert!(function_names.contains(&"INFO_DENO".to_string()));
    }
}
