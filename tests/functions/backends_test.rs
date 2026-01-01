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
}