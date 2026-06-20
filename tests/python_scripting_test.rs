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
    use std::f64::consts::{PI, TAU};

    #[test]
    fn test_python_function_creation_and_execution() {
        let db = Database::open("memory://python_create_test").unwrap();

        // Create a Python function that returns a value
        db.execute(
            r#"
            CREATE FUNCTION compute_sum(a INTEGER, b INTEGER) RETURNS INTEGER
            LANGUAGE PYTHON AS 'return arguments[0] + arguments[1]'
        "#,
            (),
        )
        .unwrap();

        // Execute the function
        let result: i64 = db.query_one("SELECT compute_sum(10, 20)", ()).unwrap();
        assert_eq!(result, 30);
    }

    #[test]
    fn test_python_function_with_different_types() {
        let db = Database::open("memory://python_types_test").unwrap();

        // Test INTEGER return
        db.execute(
            r#"
            CREATE FUNCTION get_answer() RETURNS INTEGER
            LANGUAGE PYTHON AS 'return 42'
        "#,
            (),
        )
        .unwrap();

        // Test TEXT return
        db.execute(
            r#"
            CREATE FUNCTION get_message() RETURNS TEXT
            LANGUAGE PYTHON AS 'return "Hello from Python"'
        "#,
            (),
        )
        .unwrap();

        // Test BOOLEAN return
        db.execute(
            r#"
            CREATE FUNCTION get_truth() RETURNS BOOLEAN
            LANGUAGE PYTHON AS 'return True'
        "#,
            (),
        )
        .unwrap();

        let int_result: i64 = db.query_one("SELECT get_answer()", ()).unwrap();
        assert_eq!(int_result, 42);

        let text_result: String = db.query_one("SELECT get_message()", ()).unwrap();
        assert_eq!(text_result, "Hello from Python");

        let bool_result: bool = db.query_one("SELECT get_truth()", ()).unwrap();
        assert!(bool_result);
    }

    #[test]
    fn test_python_function_with_multiple_arguments() {
        let db = Database::open("memory://python_multi_args_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION format_person(name TEXT, age INTEGER, active BOOLEAN) RETURNS TEXT
            LANGUAGE PYTHON AS 'return f"{arguments[0]} is {arguments[1]} years old, active: {arguments[2]}"'
        "#, ()).unwrap();

        let result: String = db
            .query_one("SELECT format_person('Alice', 30, true)", ())
            .unwrap();
        assert_eq!(result, "Alice is 30 years old, active: True");
    }

    #[test]
    fn test_python_function_drop() {
        let db = Database::open("memory://python_drop_test").unwrap();

        // Create function
        db.execute(
            r#"
            CREATE FUNCTION temp_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'return 123'
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
    fn test_python_function_without_return() {
        let db = Database::open("memory://python_no_result_test").unwrap();

        // Function that doesn't return a value should return NULL
        db.execute(
            r#"
            CREATE FUNCTION no_return_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'x = 42'
        "#,
            (),
        )
        .unwrap();

        // Execution should return NULL
        let result: Option<i64> = db.query_one("SELECT no_return_func()", ()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_python_string_manipulation() {
        let db = Database::open("memory://python_string_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION greet(name TEXT)
            RETURNS TEXT
            LANGUAGE PYTHON AS 'return f"Hello, {arguments[0]}!"'
        "#,
            (),
        )
        .unwrap();

        let result: String = db.query_one("SELECT greet('World')", ()).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_python_math_operations() {
        let db = Database::open("memory://python_math_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION power(base INTEGER, exp INTEGER)
            RETURNS INTEGER
            LANGUAGE PYTHON AS 'return arguments[0] ** arguments[1]'
        "#,
            (),
        )
        .unwrap();

        let result: i64 = db.query_one("SELECT power(2, 3)", ()).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_python_type_conversion() {
        let db = Database::open("memory://python_types_test").unwrap();

        // Test INTEGER
        db.execute(
            r#"
            CREATE FUNCTION double_int(x INTEGER)
            RETURNS INTEGER
            LANGUAGE PYTHON AS 'return x * 2'
        "#,
            (),
        )
        .unwrap();

        // Test FLOAT
        db.execute(
            r#"
            CREATE FUNCTION double_float(x FLOAT)
            RETURNS FLOAT
            LANGUAGE PYTHON AS 'return arguments[0] * 2.0'
        "#,
            (),
        )
        .unwrap();

        // Test BOOLEAN
        db.execute(
            r#"
            CREATE FUNCTION negate_bool(x BOOLEAN)
            RETURNS BOOLEAN
            LANGUAGE PYTHON AS 'return not arguments[0]'
        "#,
            (),
        )
        .unwrap();

        let int_result: i64 = db.query_one("SELECT double_int(21)", ()).unwrap();
        assert_eq!(int_result, 42);

        let float_result: f64 = db.query_one("SELECT double_float(3.14)", ()).unwrap();
        assert!((float_result - TAU).abs() < 0.01);

        let bool_result: bool = db.query_one("SELECT negate_bool(true)", ()).unwrap();
        assert!(!bool_result);
    }

    #[test]
    fn test_python_invalid_syntax() {
        let db = Database::open("memory://python_invalid_test").unwrap();

        // Test that invalid Python syntax fails during execution (not creation)
        db.execute(
            r#"
            CREATE FUNCTION invalid_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'return 1 +'
        "#,
            (),
        )
        .unwrap(); // Creation succeeds

        // Execution should fail
        let result: Result<i64, _> = db.query_one("SELECT invalid_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_python_runtime_error() {
        let db = Database::open("memory://python_runtime_error_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION error_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'return 1 / 0'
        "#,
            (),
        )
        .unwrap();

        let result: Result<i64, _> = db.query_one("SELECT error_func()", ());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Check that it contains some error indication
        assert!(
            err_msg.contains("error")
                || err_msg.contains("Error")
                || err_msg.contains("ZeroDivisionError")
        );
        // Should include Python exception details
    }

    #[test]
    fn test_python_syntax_validation() {
        let db = Database::open("memory://python_syntax_test").unwrap();

        // Test that valid syntax works
        let result = db.execute(
            r#"
            CREATE FUNCTION valid_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'return 1'
        "#,
            (),
        );
        assert!(result.is_ok());

        // Test that invalid syntax fails during execution
        db.execute(
            r#"
            CREATE FUNCTION invalid_func() RETURNS INTEGER
            LANGUAGE PYTHON AS 'return 1 + (invalid_syntax'
        "#,
            (),
        )
        .unwrap(); // Creation succeeds

        // Execution should fail
        let result: Result<i64, _> = db.query_one("SELECT invalid_func()", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_python_security_isolation() {
        let db = Database::open("memory://python_security_test").unwrap();

        // Note: Python backend currently does not implement security restrictions
        // These tests verify that dangerous operations can be attempted (but may succeed)
        // TODO: Implement proper sandboxing for Python backend

        // Test that file system access works (currently not restricted)
        let result = db.execute(
            r#"
            CREATE FUNCTION file_access() RETURNS TEXT
            LANGUAGE PYTHON AS 'try:
    with open("/dev/null", "r") as f: return "File access allowed"
except:
    return "File access failed"'
        "#,
            (),
        );
        if result.is_ok() {
            let exec_result: Result<String, _> = db.query_one("SELECT file_access()", ());
            // Currently, this may succeed - security restrictions not implemented yet
            let _ = exec_result; // Just check it doesn't crash
        }

        // Test that subprocess creation works (currently not restricted)
        let result = db.execute(
            r#"
            CREATE FUNCTION subprocess_test() RETURNS TEXT
            LANGUAGE PYTHON AS 'try:
    import subprocess
    return "Subprocess available"
except:
    return "Subprocess blocked"'
        "#,
            (),
        );
        if result.is_ok() {
            let exec_result: Result<String, _> = db.query_one("SELECT subprocess_test()", ());
            // Currently, this may succeed - security restrictions not implemented yet
            let _ = exec_result; // Just check it doesn't crash
        }
    }

    #[test]
    fn test_python_basic_execution() {
        let db = Database::open("memory://python_exec_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION is_even(n INTEGER)
            RETURNS BOOLEAN
            LANGUAGE PYTHON AS 'return arguments[0] % 2 == 0'
        "#,
            (),
        )
        .unwrap();

        let result: bool = db.query_one("SELECT is_even(4)", ()).unwrap();
        assert!(result);

        let result: bool = db.query_one("SELECT is_even(5)", ()).unwrap();
        assert!(!result);
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

        let result: String = db
            .query_one("SELECT process_args(42, 3.14, 'hello', true)", ())
            .unwrap();
        assert_eq!(result, "42, 3.14, hello, True");
    }

    #[test]
    fn test_python_return_types() {
        let db = Database::open("memory://python_return_test").unwrap();

        // Test all supported return type conversions
        db.execute(
            r#"
            CREATE FUNCTION return_int() RETURNS INTEGER
            LANGUAGE PYTHON AS 'return 42'
        "#,
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE FUNCTION return_float() RETURNS FLOAT
            LANGUAGE PYTHON AS 'return 3.14'
        "#,
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE FUNCTION return_text() RETURNS TEXT
            LANGUAGE PYTHON AS 'return "hello"'
        "#,
            (),
        )
        .unwrap();

        db.execute(
            r#"
            CREATE FUNCTION return_bool() RETURNS BOOLEAN
            LANGUAGE PYTHON AS 'return True'
        "#,
            (),
        )
        .unwrap();

        let int_result: i64 = db.query_one("SELECT return_int()", ()).unwrap();
        assert_eq!(int_result, 42);

        let float_result: f64 = db.query_one("SELECT return_float()", ()).unwrap();
        assert!((float_result - PI).abs() < 0.01);

        let text_result: String = db.query_one("SELECT return_text()", ()).unwrap();
        assert_eq!(text_result, "hello");

        let bool_result: bool = db.query_one("SELECT return_bool()", ()).unwrap();
        assert!(bool_result);
    }

    #[test]
    fn test_python_stored_procedure_logging() {
        use tracing_subscriber::layer::SubscriberExt;
        let (log_tx, log_rx) = crossbeam_channel::bounded(100);
        let db = Database::open("memory://python_logging_test").unwrap();
        let _shutdown = oxibase::common::logging::start_log_flusher(db.engine().clone(), log_rx);

        let layer = oxibase::common::logging::InternalLogLayer::new(log_tx);
        let _guard = tracing::subscriber::set_default(tracing_subscriber::registry().with(layer));

        db.execute(
            r#"
            CREATE PROCEDURE log_proc_test_py()
            LANGUAGE python
            AS '
import oxibase
oxibase.log("warn", "test logging from python sp")
';
        "#,
            (),
        )
        .unwrap();

        db.execute("CALL log_proc_test_py();", ()).unwrap();

        // Give flusher a bit of time
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Query the system.logs table
        let results = db
            .query("SELECT level, target, message FROM system.logs WHERE LOWER(target) = 'log_proc_test_py';", ())
            .unwrap();

        let mut found = false;
        for row_res in results {
            let row = row_res.unwrap();
            let level: String = row.get(0).unwrap();
            let target: String = row.get(1).unwrap();
            let message: String = row.get(2).unwrap();

            if level == "WARN"
                && target.to_lowercase() == "log_proc_test_py"
                && message == "test logging from python sp"
            {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "Did not find expected log entry from Python stored procedure"
        );

        // Clean up log flusher
        _shutdown.0.store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = _shutdown.1.join();
    }

    #[test]
    fn test_python_database_debugger() {
        use oxibase::common::debug::{DebugController, ResumeAction};
        use std::collections::HashMap;
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        let db = Database::open("memory://python_debug_db").unwrap();

        db.execute(
            r#"
            CREATE PROCEDURE debug_python_proc()
            LANGUAGE python
            AS '
x = 10
y = 20
x = x + 10
y = y + 20
';
            "#,
            (),
        )
        .unwrap();

        let dc = Arc::new(DebugController::new());
        // Set breakpoint on line 3 (corresponds to "x = x + 10")
        dc.set_breakpoints("DEBUG_PYTHON_PROC", vec![3]);

        let db_clone = db.clone();
        let dc_clone = Arc::clone(&dc);

        let handle = thread::spawn(move || {
            oxibase::functions::context::with_http_headers_and_debug(
                HashMap::new(),
                Some(dc_clone),
                || {
                    db_clone.execute("CALL debug_python_proc();", ()).unwrap();
                },
            );
        });

        // Let execution reach the breakpoint
        thread::sleep(Duration::from_millis(150));

        // Verify that the thread is paused and we can see local variables
        {
            let state = dc.pause_mutex.lock().unwrap();
            assert!(
                state.is_paused,
                "Python execution should be paused at the breakpoint"
            );

            if let Some(ref locals) = state.current_locals {
                assert_eq!(locals["x"], "10");
            } else {
                panic!("No local variables captured at Python breakpoint");
            }
        }

        // Issue a StepOver command
        dc.resume(ResumeAction::StepOver);

        // Give it a moment to step and pause on the next line (line 4)
        thread::sleep(Duration::from_millis(150));

        // Verify that the thread is paused at line 4 (even though there is no breakpoint there)
        {
            let state = dc.pause_mutex.lock().unwrap();
            assert!(
                state.is_paused,
                "Python execution should be paused after StepOver"
            );

            if let Some(ref locals) = state.current_locals {
                assert_eq!(locals["x"], "10");
                assert_eq!(locals["y"], "20");
            } else {
                panic!("No local variables captured after Python StepOver");
            }
        }

        // Resume execution with Continue
        dc.resume(ResumeAction::Continue);

        // Join thread
        handle.join().unwrap();
    }

    #[test]
    fn test_python_json_marshalling() {
        let db = Database::open("memory://python_json_marshall_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION process_json(j JSON) RETURNS JSON
            LANGUAGE PYTHON AS '
                # Assert it is a dict
                if not isinstance(arguments[0], dict):
                    raise TypeError("Expected dict")
                
                # Mutate it and return
                j = arguments[0]
                j["processed"] = True
                return j
            '
        "#,
            (),
        )
        .unwrap();

        let result: String = db
            .query_one("SELECT process_json(CAST('{\"key\": 42}' AS JSON))", ())
            .unwrap();

        assert!(result.contains("\"processed\": true") || result.contains("\"processed\":true"));
        assert!(result.contains("\"key\": 42") || result.contains("\"key\":42"));
    }

    #[test]
    fn test_python_timestamp_marshalling() {
        let db = Database::open("memory://python_timestamp_marshall_test").unwrap();

        db.execute(
            r#"
            CREATE FUNCTION process_timestamp(ts TIMESTAMP) RETURNS TIMESTAMP
            LANGUAGE PYTHON AS '
                # Currently rustpython-stdlib is not included, so datetime falls back to string
                if not isinstance(arguments[0], str):
                    raise TypeError("Expected str fallback for timestamp")
                
                ts_str = arguments[0]
                # Dummy manipulation: return the same string
                return ts_str
            '
        "#,
            (),
        )
        .unwrap();

        let past = chrono::Utc::now() - chrono::Duration::days(1);
        let query = format!(
            "SELECT process_timestamp(CAST('{}' AS TIMESTAMP))",
            past.to_rfc3339()
        );
        let result: oxibase::Value = db.query_one(&query, ()).unwrap();
        let dt = if let oxibase::Value::Timestamp(t) = result {
            t
        } else {
            panic!("Expected timestamp")
        };

        let diff = dt - past;
        assert_eq!(diff.num_seconds(), 0);
    }
}
