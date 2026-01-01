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
        db.execute(r#"
            CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
            RETURNS INTEGER
            LANGUAGE DENO AS 'return arguments[0] + arguments[1];'
        "#, ()).unwrap();

        let result: i64 = db.query_one("SELECT add_numbers(5, 3)", ()).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_deno_string_manipulation() {
        let db = Database::open("memory://deno_string_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION greet(name TEXT)
            RETURNS TEXT
            LANGUAGE DENO AS 'return `Hello, ${arguments[0]}!`;'
        "#, ()).unwrap();

        let result: String = db.query_one("SELECT greet('World')", ()).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_deno_math_operations() {
        let db = Database::open("memory://deno_math_test").unwrap();

        db.execute(r#"
            CREATE FUNCTION power(base INTEGER, exp INTEGER)
            RETURNS INTEGER
            LANGUAGE DENO AS 'return Math.pow(arguments[0], arguments[1]);'
        "#, ()).unwrap();

        let result: i64 = db.query_one("SELECT power(2, 3)", ()).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_deno_type_conversion() {
        let db = Database::open("memory://deno_types_test").unwrap();

        // Test INTEGER
        db.execute(r#"
            CREATE FUNCTION double_int(x INTEGER)
            RETURNS INTEGER
            LANGUAGE DENO AS 'return x * 2;'
        "#, ()).unwrap();

        // Test FLOAT
        db.execute(r#"
            CREATE FUNCTION double_float(x FLOAT)
            RETURNS FLOAT
            LANGUAGE DENO AS 'return arguments[0] * 2.0;'
        "#, ()).unwrap();

        // Test BOOLEAN
        db.execute(r#"
            CREATE FUNCTION negate_bool(x BOOLEAN)
            RETURNS BOOLEAN
            LANGUAGE DENO AS 'return !arguments[0];'
        "#, ()).unwrap();

        let int_result: i64 = db.query_one("SELECT double_int(21)", ()).unwrap();
        assert_eq!(int_result, 42);

        let float_result: f64 = db.query_one("SELECT double_float(3.14)", ()).unwrap();
        assert!((float_result - 6.28).abs() < 0.01);

        let bool_result: bool = db.query_one("SELECT negate_bool(true)", ()).unwrap();
        assert_eq!(bool_result, false);
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
        assert!(!result.contains("blocked") || result.contains("Fetch works") || result.contains("network"));
    }
}