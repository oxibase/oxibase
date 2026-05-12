    fn execute_row_triggers(
        &self,
        table_name: &str,
        timing: &str,
        event: &str,
        new_row: Option<&mut crate::core::Row>,
        old_row: Option<&crate::core::Row>,
        schema: &crate::core::Schema,
    ) -> Result<()> {
        let registry = &self.trigger_registry;
        let triggers = match (timing, event) {
            ("BEFORE", "INSERT") => registry.get_before_insert(table_name),
            ("AFTER", "INSERT") => registry.get_after_insert(table_name),
            ("BEFORE", "UPDATE") => registry.get_before_update(table_name),
            ("AFTER", "UPDATE") => registry.get_after_update(table_name),
            ("BEFORE", "DELETE") => registry.get_before_delete(table_name),
            ("AFTER", "DELETE") => registry.get_after_delete(table_name),
            _ => Vec::new(),
        };

        if triggers.is_empty() {
            return Ok(());
        }

        crate::functions::backends::triggers::with_trigger_context(new_row, old_row, schema, || {
            for trigger in triggers {
                if let Some(backend) = self.function_registry.get_backend(&trigger.language) {
                    // For triggers we pass empty arguments to the main script since NEW and OLD are injected via proxies
                    if let Err(e) = backend.execute(&trigger.code, &[], &[]) {
                        return Err(crate::core::Error::internal(format!("Trigger execution failed: {}", e)));
                    }
                } else {
                    return Err(crate::core::Error::internal(format!("Unsupported trigger language: {}", trigger.language)));
                }
            }
            Ok(())
        })
    }
