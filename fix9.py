import re

with open("src/executor/mod.rs", "r") as f:
    content = f.read()

# Fix the duplicate with_function_registry
content = re.sub(r'    pub fn with_function_registry\(\n        engine: Arc<MVCCEngine>,\n        function_registry: Arc<FunctionRegistry>,\n    \) -> Self \{\n        let executor = Self \{\n            engine,\n            function_registry,\n            default_isolation_level: crate::core::IsolationLevel::ReadCommitted,\n            query_cache: QueryCache::default\(\),\n            semantic_cache: SemanticCache::default\(\),\n            active_transaction: Mutex::new\(None\),\n            query_planner: std::sync::OnceLock::new\(\),\n            trigger_registry: Arc::new\(triggers::TriggerRegistry::new\(\)\),\n        \};\n\n        // Initialize system schema and tables\n        let _ = executor\.ensure_system_schema_and_migrations\(\);\n\n        // Load user-defined functions from system table\n        let _ = executor\.load_functions\(\);\n        let _ = executor\.load_procedures\(\);\n        let _ = executor\.load_triggers\(\);\n\n        executor\n    \}\n\n    /// Create a new executor with a custom function registry\n', "    /// Create a new executor with a custom function registry\n", content)

with open("src/executor/mod.rs", "w") as f:
    f.write(content)
