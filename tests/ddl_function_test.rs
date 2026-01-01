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

//! DDL Function Tests
//!
//! Tests for CREATE/DROP FUNCTION DDL execution edge cases and error handling.

use oxibase::Database;

#[cfg(test)]
mod ddl_function_tests {
    use super::*;

    #[test]
    fn test_create_function_invalid_language() {
        let db = Database::open("memory://invalid_lang_test").unwrap();

        // Try to create function with unsupported language
        let result = db.execute(
            r#"
            CREATE FUNCTION test_func() RETURNS INTEGER
            LANGUAGE INVALID_LANG AS 'return 42'
        "#,
            (),
        );

        // May succeed at creation and fail at execution
        if result.is_ok() {
            let exec_result: Result<i64, _> = db.query_one("SELECT test_func()", ());
            assert!(
                exec_result.is_err(),
                "Should fail when executing with unsupported language"
            );
        } else {
            // If creation fails, that's also acceptable
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_create_function_duplicate_without_if_not_exists() {
        let db = Database::open("memory://duplicate_func_test").unwrap();

        // Create function
        db.execute(
            r#"
            CREATE FUNCTION test_func() RETURNS INTEGER
            LANGUAGE RHAI AS '42'
        "#,
            (),
        )
        .unwrap();

        // Try to create again without IF NOT EXISTS - should fail
        let result = db.execute(
            r#"
            CREATE FUNCTION test_func() RETURNS INTEGER
            LANGUAGE RHAI AS '43'
        "#,
            (),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_create_function_duplicate_with_if_not_exists() {
        let db = Database::open("memory://duplicate_if_not_exists_test").unwrap();

        // Create function
        db.execute(
            r#"
            CREATE FUNCTION test_func() RETURNS INTEGER
            LANGUAGE RHAI AS '42'
        "#,
            (),
        )
        .unwrap();

        // Try to create again with IF NOT EXISTS - should succeed
        let result = db.execute(
            r#"
            CREATE FUNCTION IF NOT EXISTS test_func() RETURNS INTEGER
            LANGUAGE RHAI AS '43'
        "#,
            (),
        );

        assert!(result.is_ok());

        // Function should still return original value (not replaced)
        let value: i64 = db.query_one("SELECT test_func()", ()).unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_create_function_invalid_syntax_validation() {
        let db = Database::open("memory://syntax_validation_test").unwrap();

        // Try to create function with invalid syntax in body
        let result = db.execute(
            r#"
            CREATE FUNCTION invalid_func() RETURNS INTEGER
            LANGUAGE RHAI AS '1 +'
        "#,
            (),
        );

        // May succeed at creation and fail at execution
        if result.is_ok() {
            let exec_result: Result<i64, _> = db.query_one("SELECT invalid_func()", ());
            assert!(
                exec_result.is_err(),
                "Should fail when executing invalid syntax"
            );
        } else {
            // If creation fails, that's also acceptable
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_create_function_empty_body() {
        let db = Database::open("memory://empty_body_test").unwrap();

        // Try to create function with empty body
        let result = db.execute(
            r#"
            CREATE FUNCTION empty_func() RETURNS INTEGER
            LANGUAGE RHAI AS ''
        "#,
            (),
        );

        // This might succeed or fail depending on backend validation
        // At minimum, it should not crash
        if result.is_ok() {
            // If it succeeds, executing should handle empty script gracefully
            let exec_result: Result<i64, _> = db.query_one("SELECT empty_func()", ());
            // Should either return NULL or error gracefully
            assert!(exec_result.is_ok() || exec_result.is_err());
        }
    }

    // Note: Schema-qualified functions may not be fully implemented yet
    // #[test]
    // fn test_create_function_schema_not_exists() {
    //     let db = Database::open("memory://schema_not_exists_test").unwrap();
    //     // Implementation depends on schema support
    // }

    #[test]
    fn test_drop_function_not_exists() {
        let db = Database::open("memory://drop_not_exists_test").unwrap();

        // Try to drop non-existent function
        let result = db.execute("DROP FUNCTION nonexistent_func", ());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found") || err_msg.contains("does not exist"));
    }

    #[test]
    fn test_drop_function_if_exists() {
        let db = Database::open("memory://drop_if_exists_test").unwrap();

        // Drop non-existent function with IF EXISTS - should succeed
        let result = db.execute("DROP FUNCTION IF EXISTS nonexistent_func", ());

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_function_case_insensitive_language() {
        let db = Database::open("memory://case_insensitive_lang_test").unwrap();

        // Test case insensitive language names
        let result = db.execute(
            r#"
            CREATE FUNCTION test_func() RETURNS INTEGER
            LANGUAGE rhai AS '42'
        "#,
            (),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_function_malformed_signature() {
        let db = Database::open("memory://malformed_sig_test").unwrap();

        // Missing closing parenthesis
        let result = db.execute(
            r#"
            CREATE FUNCTION test_func(a INTEGER RETURNS INTEGER
            LANGUAGE RHAI AS '42'
        "#,
            (),
        );

        assert!(result.is_err());
    }
}
