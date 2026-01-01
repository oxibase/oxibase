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

//! Backend registry tests
//!
//! Tests for the scripting backend registry functionality.

use oxibase::functions::backends::{create_backend_registry, BackendRegistry};

#[cfg(test)]
mod backend_registry_tests {
    use super::*;

    #[test]
    fn test_backend_registry_creation() {
        let registry = create_backend_registry();

        // Should always have Rhai backend
        assert!(registry.is_language_supported("rhai"));

        // Check that registry has expected backends
        let languages = registry.list_supported_languages();
        assert!(languages.contains(&"rhai".to_string()));

        #[cfg(feature = "deno")]
        assert!(languages.contains(&"deno".to_string()));

        #[cfg(feature = "python")]
        assert!(languages.contains(&"python".to_string()));
    }

    #[test]
    fn test_backend_registry_get_backend() {
        let registry = create_backend_registry();

        // Test getting Rhai backend
        let rhai_backend = registry.get_backend("rhai");
        assert!(rhai_backend.is_some());
        assert_eq!(rhai_backend.unwrap().name(), "rhai");

        // Test case insensitive
        let rhai_backend_upper = registry.get_backend("RHAI");
        assert!(rhai_backend_upper.is_some());

        // Test unsupported language
        let unsupported = registry.get_backend("unsupported");
        assert!(unsupported.is_none());
    }

    #[test]
    fn test_backend_registry_language_support() {
        let registry = create_backend_registry();

        // Rhai should always be supported
        assert!(registry.is_language_supported("rhai"));
        assert!(registry.is_language_supported("RHAI"));

        // Unsupported language
        assert!(!registry.is_language_supported("lua"));
        assert!(!registry.is_language_supported("ruby"));
    }

    #[test]
    fn test_backend_registry_empty() {
        let registry = BackendRegistry::new();

        // Empty registry should not support any languages
        assert!(!registry.is_language_supported("rhai"));
        assert!(registry.list_supported_languages().is_empty());
        assert!(registry.get_backend("any").is_none());
    }

    #[test]
    fn test_backend_registry_duplicate_registration() {
        use oxibase::functions::backends::rhai::RhaiBackend;
        use std::sync::Arc;

        let mut registry = BackendRegistry::new();

        // Register Rhai backend
        let rhai_backend = Arc::new(RhaiBackend::new());
        registry.register_backend(rhai_backend.clone());

        // Register again (should be fine, just overwrites)
        registry.register_backend(rhai_backend);

        // Should still work
        assert!(registry.is_language_supported("rhai"));
        let backend = registry.get_backend("rhai").unwrap();
        assert_eq!(backend.name(), "rhai");
    }

    #[test]
    fn test_backend_registry_unsupported_language_creation() {
        let db = oxibase::Database::open("memory://unsupported_lang_test").unwrap();

        // Try to create function with unsupported language
        let result = db.execute(r#"
            CREATE FUNCTION test_func() RETURNS INTEGER
            LANGUAGE UNSUPPORTED AS 'return 42'
        "#, ());

        // Should fail because language is not supported
        assert!(result.is_err());
    }

    #[test]
    fn test_rhai_backend_named_parameters() {
        use oxibase::functions::backends::rhai::RhaiBackend;
        use oxibase::core::{Value, Result};

        let backend = RhaiBackend::new();

        // Test with named parameters
        let result = backend.execute("a + b", &[Value::Integer(3), Value::Integer(4)], &["a".to_string(), "b".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(7));
    }

    #[test]
    fn test_rhai_backend_wrong_param_names() {
        use oxibase::functions::backends::rhai::RhaiBackend;
        use oxibase::core::Value;

        let backend = RhaiBackend::new();

        // Test with wrong param names (should fail)
        let result = backend.execute("x + y", &[Value::Integer(3), Value::Integer(4)], &["a".to_string(), "b".to_string()]);
        assert!(result.is_err()); // Rhai will fail because x and y are undefined
    }

    #[test]
    fn test_rhai_backend_multiple_types() {
        use oxibase::functions::backends::rhai::RhaiBackend;
        use oxibase::core::Value;

        let backend = RhaiBackend::new();

        // Test with different types: integer, float, text, boolean
        let result = backend.execute("i + f + (if b { 10 } else { 0 })", &[Value::Integer(5), Value::Float(3.5), Value::Boolean(true)], &["i".to_string(), "f".to_string(), "b".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(18.5)); // 5 + 3.5 + 10
    }

    #[test]
    fn test_rhai_backend_complex_expression() {
        use oxibase::functions::backends::rhai::RhaiBackend;
        use oxibase::core::Value;

        let backend = RhaiBackend::new();

        // Test complex expression with multiple params
        let result = backend.execute("if x > y { x * 2 } else { y / 2 }", &[Value::Integer(10), Value::Integer(4)], &["x".to_string(), "y".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(20)); // 10 > 4, so 10 * 2
    }

    #[test]
    fn test_rhai_backend_string_manipulation() {
        use oxibase::functions::backends::rhai::RhaiBackend;
        use oxibase::core::Value;

        let backend = RhaiBackend::new();

        // Test string concatenation
        let result = backend.execute("name + \" is \" + age + \" years old\"", &[Value::Text("Alice".into()), Value::Integer(30)], &["name".to_string(), "age".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Text("Alice is 30 years old".into()));
    }

    #[cfg(feature = "deno")]
    #[test]
    fn test_deno_backend_named_parameters() {
        use oxibase::functions::backends::deno::DenoBackend;
        use oxibase::core::Value;

        let backend = DenoBackend::new();

        // Test basic named parameters
        let result = backend.execute("return a + b;", &[Value::Integer(3), Value::Integer(4)], &["a".to_string(), "b".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(7));
    }

    #[cfg(feature = "deno")]
    #[test]
    fn test_deno_backend_multiple_types() {
        use oxibase::functions::backends::deno::DenoBackend;
        use oxibase::core::Value;

        let backend = DenoBackend::new();

        // Test with different types
        let result = backend.execute("return i + f + (b ? 1 : 0);", &[Value::Integer(5), Value::Float(2.5), Value::Boolean(true)], &["i".to_string(), "f".to_string(), "b".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(8.5));
    }

    #[cfg(feature = "python")]
    #[test]
    fn test_python_backend_named_parameters() {
        use oxibase::functions::backends::python::PythonBackend;
        use oxibase::core::Value;

        let backend = PythonBackend::new();

        // Test basic named parameters
        let result = backend.execute("return a + b", &[Value::Integer(3), Value::Integer(4)], &["a".to_string(), "b".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(7));
    }

    #[cfg(feature = "python")]
    #[test]
    fn test_python_backend_string_ops() {
        use oxibase::functions::backends::python::PythonBackend;
        use oxibase::core::Value;

        let backend = PythonBackend::new();

        // Test string operations
        let result = backend.execute("return name.upper() + str(age)", &[Value::Text("alice".into()), Value::Integer(25)], &["name".to_string(), "age".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Text("ALICE25".into()));
    }
}