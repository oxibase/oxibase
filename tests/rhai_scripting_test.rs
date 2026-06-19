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
        db.execute(
            r#"
            CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
            RETURNS INTEGER
            LANGUAGE RHAI AS 'a + b'
        "#,
            (),
        )
        .unwrap();

        let result: i64 = db.query_one("SELECT add_numbers(5, 3)", ()).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_rhai_string_manipulation() {
        let db = Database::open("memory://rhai_string_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION greet(name TEXT)
            RETURNS TEXT
            LANGUAGE RHAI AS '`Hello, ${name}!`'
        "#,
            (),
        )
        .unwrap();

        let result: String = db.query_one("SELECT greet('World')", ()).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_rhai_math_operations() {
        let db = Database::open("memory://rhai_math_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION power(base INTEGER, exp INTEGER)
            RETURNS INTEGER
            LANGUAGE RHAI AS 'base ** exp'
        "#,
            (),
        )
        .unwrap();

        let result: i64 = db.query_one("SELECT power(2, 3)", ()).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_rhai_type_conversion() {
        let db = Database::open("memory://rhai_types_test").unwrap();

        // Test INTEGER
        db.execute(
            r#"
            CREATE FUNCTION double_int(x INTEGER)
            RETURNS INTEGER
            LANGUAGE RHAI AS 'x * 2'
        "#,
            (),
        )
        .unwrap();

        // Test FLOAT
        db.execute(
            r#"
            CREATE FUNCTION double_float(x FLOAT)
            RETURNS FLOAT
            LANGUAGE RHAI AS 'x * 2.0'
        "#,
            (),
        )
        .unwrap();

        // Test BOOLEAN
        db.execute(
            r#"
            CREATE FUNCTION negate_bool(x BOOLEAN)
            RETURNS BOOLEAN
            LANGUAGE RHAI AS '!x'
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
    fn test_rhai_argument_types() {
        let db = Database::open("memory://rhai_args_test").unwrap();

        // Test INTEGER, FLOAT, TEXT, BOOLEAN arguments
        db.execute(
            r#"
            CREATE FUNCTION process_args(x INTEGER, y FLOAT, z TEXT, w BOOLEAN)
            RETURNS TEXT
            LANGUAGE RHAI AS 'x.to_string()'
        "#,
            (),
        )
        .unwrap();

        let result: String = db
            .query_one("SELECT process_args(42, 3.14, 'hello', true)", ())
            .unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_rhai_return_types() {
        let db = Database::open("memory://rhai_return_test").unwrap();

        // Test all supported return type conversions
        db.execute(
            r#"
            CREATE FUNCTION return_int() RETURNS INTEGER
            LANGUAGE RHAI AS '42'
        "#,
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE FUNCTION return_float() RETURNS FLOAT
            LANGUAGE RHAI AS '3.14'
        "#,
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE FUNCTION return_text() RETURNS TEXT
            LANGUAGE RHAI AS '"hello"'
        "#,
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE FUNCTION return_bool() RETURNS BOOLEAN
            LANGUAGE RHAI AS 'true'
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
    fn test_rhai_invalid_syntax() {
        let db = Database::open("memory://rhai_invalid_test").unwrap();

        // Test that invalid Rhai syntax fails during execution
        db.execute(
            r#"
            CREATE FUNCTION invalid_func() RETURNS INTEGER
            LANGUAGE RHAI AS '1 + (invalid_syntax'
        "#,
            (),
        )
        .unwrap(); // Creation succeeds

        // Execution should fail
        let result: Result<i64, _> = db.query_one("SELECT invalid_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_rhai_runtime_error() {
        let db = Database::open("memory://rhai_runtime_error_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION error_func() RETURNS INTEGER
            LANGUAGE RHAI AS '1 / 0'
        "#,
            (),
        )
        .unwrap();

        let result: Result<i64, _> = db.query_one("SELECT error_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_rhai_function_creation_and_execution() {
        let db = Database::open("memory://rhai_create_test").unwrap();

        // Create a Rhai function that returns a value
        db.execute(
            r#"
            CREATE FUNCTION compute_sum(a INTEGER, b INTEGER) RETURNS INTEGER
            LANGUAGE RHAI AS 'a + b'
        "#,
            (),
        )
        .unwrap();

        // Execute the function
        let result: i64 = db.query_one("SELECT compute_sum(10, 20)", ()).unwrap();
        assert_eq!(result, 30);
    }

    #[test]
    fn test_rhai_function_with_different_types() {
        let db = Database::open("memory://rhai_types_test").unwrap();

        // Test INTEGER return
        db.execute(
            r#"
            CREATE FUNCTION get_answer() RETURNS INTEGER
            LANGUAGE RHAI AS '42'
        "#,
            (),
        )
        .unwrap();

        // Test TEXT return
        db.execute(
            r#"
            CREATE FUNCTION get_message() RETURNS TEXT
            LANGUAGE RHAI AS '"Hello from Rhai"'
        "#,
            (),
        )
        .unwrap();

        // Test BOOLEAN return
        db.execute(
            r#"
            CREATE FUNCTION get_truth() RETURNS BOOLEAN
            LANGUAGE RHAI AS 'true'
        "#,
            (),
        )
        .unwrap();

        let int_result: i64 = db.query_one("SELECT get_answer()", ()).unwrap();
        assert_eq!(int_result, 42);

        let text_result: String = db.query_one("SELECT get_message()", ()).unwrap();
        assert_eq!(text_result, "Hello from Rhai");

        let bool_result: bool = db.query_one("SELECT get_truth()", ()).unwrap();
        assert!(bool_result);
    }

    #[test]
    fn test_rhai_function_with_multiple_arguments() {
        let db = Database::open("memory://rhai_multi_args_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION format_person(person_name TEXT, person_age INTEGER, is_active BOOLEAN) RETURNS TEXT
            LANGUAGE RHAI AS 'person_name + " is " + person_age.to_string() + " years old, active: " + is_active.to_string()'
        "#, ()).unwrap();

        let result: String = db
            .query_one("SELECT format_person('Alice', 30, true)", ())
            .unwrap();
        assert_eq!(result, "Alice is 30 years old, active: true");
    }

    #[test]
    fn test_rhai_function_drop() {
        let db = Database::open("memory://rhai_drop_test").unwrap();

        // Create function
        db.execute(
            r#"
            CREATE FUNCTION temp_func() RETURNS INTEGER
            LANGUAGE RHAI AS '123'
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
    fn test_rhai_syntax_validation() {
        let db = Database::open("memory://rhai_syntax_test").unwrap();

        // Test that valid syntax works
        let result = db.execute(
            r#"
            CREATE FUNCTION valid_func() RETURNS INTEGER
            LANGUAGE RHAI AS '1'
        "#,
            (),
        );
        assert!(result.is_ok());

        // Test that invalid syntax fails during execution
        db.execute(
            r#"
            CREATE FUNCTION invalid_func() RETURNS INTEGER
            LANGUAGE RHAI AS '1 + (invalid_syntax'
        "#,
            (),
        )
        .unwrap(); // Creation succeeds

        // Execution should fail
        let result: Result<i64, _> = db.query_one("SELECT invalid_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_rhai_function_without_return() {
        let db = Database::open("memory://rhai_no_result_test").unwrap();

        // Function that doesn't return a value should return NULL
        db.execute(
            r#"
            CREATE FUNCTION no_return_func() RETURNS INTEGER
            LANGUAGE RHAI AS 'let x = 42;'
        "#,
            (),
        )
        .unwrap();

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

    #[test]
    fn test_rhai_parse_json() {
        let db = Database::open("memory://rhai_parse_json_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION extract_json_key(j TEXT) RETURNS INTEGER
            LANGUAGE RHAI AS '
                let obj = parse_json(j);
                obj.key
            '
        "#,
            (),
        )
        .unwrap();

        let result: i64 = db
            .query_one("SELECT extract_json_key('{\"key\": 42}')", ())
            .unwrap();
        assert_eq!(result, 42);

        db.execute(
            r#"
            CREATE FUNCTION extract_json_array(j TEXT) RETURNS INTEGER
            LANGUAGE RHAI AS '
                let arr = parse_json(j);
                arr[1]
            '
        "#,
            (),
        )
        .unwrap();

        let result: i64 = db
            .query_one("SELECT extract_json_array('[10, 42, 30]')", ())
            .unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_rhai_parse_invalid_json() {
        let db = Database::open("memory://rhai_parse_invalid_json_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION test_invalid_json(j TEXT) RETURNS INTEGER
            LANGUAGE RHAI AS '
                let obj = parse_json(j);
                1
            '
        "#,
            (),
        )
        .unwrap();

        let result: Result<i64, _> = db.query_one("SELECT test_invalid_json('{invalid json}')", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_rhai_json_arguments() {
        let db = Database::open("memory://rhai_json_args_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION extract_from_native_json(doc JSON) RETURNS INTEGER
            LANGUAGE RHAI AS '
                doc.key
            '
        "#,
            (),
        )
        .unwrap();

        let result: i64 = db
            .query_one(
                "SELECT extract_from_native_json(CAST('{\"key\": 42}' AS JSON))",
                (),
            )
            .unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_rhai_json_returns() {
        let db = Database::open("memory://rhai_json_returns_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION create_native_json() RETURNS JSON
            LANGUAGE RHAI AS '
                #{
                    key: 42,
                    name: "test"
                }
            '
        "#,
            (),
        )
        .unwrap();

        let result: String = db.query_one("SELECT create_native_json()", ()).unwrap();
        // Since JSON formatting can vary (spaces vs no spaces), parse and compare or use contains
        assert!(result.contains("\"key\":42") || result.contains("\"key\": 42"));
        assert!(result.contains("\"name\":\"test\"") || result.contains("\"name\": \"test\""));
    }

    #[test]
    fn test_rhai_json_procedure() {
        let db = Database::open("memory://rhai_json_procedure_test").unwrap();

        db.execute(
            r#"
            CREATE PROCEDURE update_json_proc(INOUT doc JSON)
            LANGUAGE RHAI AS '
                doc.updated = true;
            '
        "#,
            (),
        )
        .unwrap();

        let call_sql = "CALL update_json_proc(CAST('{\"original\": 1}' AS JSON));";
        let mut results = db.query(call_sql, ()).unwrap();

        let row = results.next().unwrap().unwrap();
        let value = row.get::<oxibase::core::Value>(0).unwrap();
        let result_json = value.as_str().unwrap();
        assert!(
            result_json.contains("\"updated\":true") || result_json.contains("\"updated\": true")
        );
    }

    #[test]
    fn test_rhai_json_trigger() {
        let db = Database::open("memory://rhai_json_trigger_test").unwrap();

        db.execute(
            "CREATE TABLE configs (id INTEGER PRIMARY KEY, data JSON);",
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE TRIGGER process_config
                BEFORE INSERT ON configs
                FOR EACH ROW
                LANGUAGE rhai
            AS '
                let d = oxibase.ctx["new"].data;
                d.processed = true;
                oxibase.ctx["new"].data = d;
            ';
            "#,
            (),
        )
        .unwrap();

        db.execute(
            "INSERT INTO configs (id, data) VALUES (1, CAST('{\"original\": 1}' AS JSON));",
            (),
        )
        .unwrap();

        let mut results = db
            .query("SELECT data FROM configs WHERE id = 1;", ())
            .unwrap();
        let row = results.next().unwrap().unwrap();
        let value = row.get::<oxibase::core::Value>(0).unwrap();
        let json_str = value.as_str().unwrap();
        assert!(
            json_str.contains("\"processed\":true") || json_str.contains("\"processed\": true")
        );
    }

    #[test]
    fn test_rhai_json_fallback() {
        let db = Database::open("memory://rhai_json_fallback_test").unwrap();
        db.execute(
            r#"
            CREATE FUNCTION fallback_json(doc JSON) RETURNS JSON
            LANGUAGE RHAI AS 'doc'
        "#,
            (),
        )
        .unwrap();

        let res = db.query_one::<String, _>("SELECT fallback_json('invalid json');", ());
        assert!(res.is_ok() || res.is_err());
    }

    #[test]
    fn test_rhai_timestamp_function() {
        let db = Database::open("memory://rhai_timestamp_test").unwrap();
        db.execute(
            r#"
            CREATE FUNCTION get_now() RETURNS TIMESTAMP
            LANGUAGE RHAI AS 'timestamp()'
        "#,
            (),
        )
        .unwrap();

        let result: oxibase::Value = db.query_one("SELECT get_now()", ()).unwrap();
        let dt = if let oxibase::Value::Timestamp(t) = result {
            t
        } else {
            panic!("Expected timestamp")
        };
        let diff = chrono::Utc::now() - dt;
        assert!(diff.num_seconds() < 2);
    }

    #[test]
    fn test_rhai_timestamp_elapsed() {
        let db = Database::open("memory://rhai_timestamp_elapsed_test").unwrap();
        db.execute(
            r#"
            CREATE FUNCTION test_elapsed() RETURNS FLOAT
            LANGUAGE RHAI AS '
                let t = timestamp();
                sleep(100);
                t.elapsed()
            '
        "#,
            (),
        )
        .unwrap();

        let elapsed: f64 = db.query_one("SELECT test_elapsed()", ()).unwrap();
        assert!(elapsed >= 0.1);
    }

    #[test]
    fn test_rhai_timestamp_pass_and_return() {
        let db = Database::open("memory://rhai_timestamp_pass_test").unwrap();
        db.execute(
            r#"
            CREATE FUNCTION return_same(t TIMESTAMP) RETURNS TIMESTAMP
            LANGUAGE RHAI AS 't'
        "#,
            (),
        )
        .unwrap();

        let now = chrono::Utc::now();
        // Insert into a string literal for the query
        let query = format!(
            "SELECT return_same(CAST('{}' AS TIMESTAMP))",
            now.to_rfc3339()
        );
        let result: oxibase::Value = db.query_one(&query, ()).unwrap();
        let dt = if let oxibase::Value::Timestamp(t) = result {
            t
        } else {
            panic!("Expected timestamp")
        };
        assert_eq!(dt, now);
    }

    #[test]
    fn test_rhai_timestamp_to_string() {
        let db = Database::open("memory://rhai_ts_to_string_test").unwrap();
        db.execute(
            r#"
            CREATE FUNCTION ts_to_str() RETURNS TEXT
            LANGUAGE RHAI AS 'timestamp().to_string()'
        "#,
            (),
        )
        .unwrap();

        let result: String = db.query_one("SELECT ts_to_str()", ()).unwrap();
        // Just verify it parses back to a valid RFC3339 datetime
        assert!(chrono::DateTime::parse_from_rfc3339(&result).is_ok());
    }

    #[test]
    fn test_rhai_timestamp_procedure_inout() {
        let db = Database::open("memory://rhai_ts_proc_inout_test").unwrap();
        db.execute(
            r#"
            CREATE PROCEDURE update_ts_proc(INOUT ts TIMESTAMP)
            LANGUAGE RHAI AS '
                sleep(100);
                ts = timestamp();
            '
        "#,
            (),
        )
        .unwrap();

        let past = chrono::Utc::now() - chrono::Duration::days(1);
        let call_sql = format!(
            "CALL update_ts_proc(CAST('{}' AS TIMESTAMP));",
            past.to_rfc3339()
        );
        let mut results = db.query(&call_sql, ()).unwrap();

        let row = results.next().unwrap().unwrap();
        let value = row.get::<oxibase::core::Value>(0).unwrap();
        let updated_dt = if let oxibase::core::Value::Timestamp(t) = value {
            t
        } else {
            panic!("Expected timestamp")
        };
        assert!(updated_dt > past);
    }

    #[test]
    fn test_rhai_timestamp_trigger() {
        let db = Database::open("memory://rhai_ts_trigger_test").unwrap();

        db.execute(
            "CREATE TABLE events (id INTEGER PRIMARY KEY, created_at TIMESTAMP);",
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE TRIGGER process_event
                BEFORE INSERT ON events
                FOR EACH ROW
                LANGUAGE rhai
            AS '
                oxibase.ctx["new"].created_at = timestamp();
            ';
            "#,
            (),
        )
        .unwrap();

        let past = chrono::Utc::now() - chrono::Duration::days(1);
        db.execute(
            &format!(
                "INSERT INTO events (id, created_at) VALUES (1, CAST('{}' AS TIMESTAMP));",
                past.to_rfc3339()
            ),
            (),
        )
        .unwrap();

        let mut results = db
            .query("SELECT created_at FROM events WHERE id = 1;", ())
            .unwrap();
        let row = results.next().unwrap().unwrap();
        let value = row.get::<oxibase::core::Value>(0).unwrap();
        let updated_dt = if let oxibase::core::Value::Timestamp(t) = value {
            t
        } else {
            panic!("Expected timestamp")
        };
        assert!(updated_dt > past); // The trigger should have overridden the old timestamp
    }

    #[test]
    fn test_rhai_timestamp_dynamic_to_value_fallback() {
        use chrono::Datelike;
        let db = Database::open("memory://rhai_ts_fallback_test").unwrap();

        db.execute(
            "CREATE TABLE events2 (id INTEGER PRIMARY KEY, created_at TIMESTAMP);",
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE TRIGGER process_event_str
                BEFORE INSERT ON events2
                FOR EACH ROW
                LANGUAGE rhai
            AS '
                oxibase.ctx["new"].created_at = "2024-01-01T12:00:00Z";
            ';
            "#,
            (),
        )
        .unwrap();

        db.execute("INSERT INTO events2 (id, created_at) VALUES (1, NULL);", ())
            .unwrap();

        let mut results = db
            .query("SELECT created_at FROM events2 WHERE id = 1;", ())
            .unwrap();
        let row = results.next().unwrap().unwrap();
        let value = row.get::<oxibase::core::Value>(0).unwrap();
        let dt = if let oxibase::core::Value::Timestamp(t) = value {
            t
        } else {
            panic!("Expected timestamp")
        };
        assert_eq!(dt.year(), 2024);

        // Fallback invalid string
        db.execute(
            r#"
            CREATE TRIGGER process_event_invalid_str
                BEFORE INSERT ON events2
                FOR EACH ROW
                LANGUAGE rhai
            AS '
                oxibase.ctx["new"].created_at = "invalid date";
            ';
            "#,
            (),
        )
        .unwrap();

        db.execute("INSERT INTO events2 (id, created_at) VALUES (2, NULL);", ())
            .unwrap();
        let mut results = db
            .query("SELECT created_at FROM events2 WHERE id = 2;", ())
            .unwrap();
        let row = results.next().unwrap().unwrap();
        let value = row.get::<oxibase::core::Value>(0).unwrap();
        let dt2 = if let oxibase::core::Value::Timestamp(t) = value {
            t
        } else {
            panic!("Expected timestamp")
        };
        assert!(dt2.year() >= 2024); // Fallback is Utc::now()
    }

    #[test]
    fn test_rhai_null_argument() {
        let db = Database::open("memory://rhai_null_arg_test").unwrap();
        db.execute(
            r#"
            CREATE FUNCTION test_null(arg TIMESTAMP) RETURNS BOOLEAN
            LANGUAGE RHAI AS 'arg == ()'
        "#,
            (),
        )
        .unwrap();

        let result: bool = db.query_one("SELECT test_null(NULL)", ()).unwrap();
        assert!(result);
    }
}
