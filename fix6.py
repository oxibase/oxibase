with open("src/executor/mod.rs", "r") as f:
    content = f.read()

# Add to with_function_registry
content = content.replace("        // Note: We ignore errors during startup to avoid failing if the table doesn't exist yet\n        let _ = executor.load_functions();", 
                          "        let _ = executor.ensure_system_schema_and_migrations();\n\n        // Note: We ignore errors during startup to avoid failing if the table doesn't exist yet\n        let _ = executor.load_functions();")

with open("src/executor/mod.rs", "w") as f:
    f.write(content)
