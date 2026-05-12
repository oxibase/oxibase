    /// Load triggers from system table
    fn load_triggers(&self) -> Result<()> {
        use crate::storage::triggers::SYS_TRIGGERS;

        let tx = self.engine.begin_transaction()?;
        let tables = tx.list_tables()?;
        let has_triggers_table = tables
            .iter()
            .any(|t| t.eq_ignore_ascii_case(SYS_TRIGGERS));

        if !has_triggers_table {
            return Ok(());
        }

        let table = tx.get_table(SYS_TRIGGERS)?;
        let mut scanner = table.scan(&[], None)?;

        let mut triggers = Vec::new();
        while scanner.next() {
            let row = scanner.row();
            if let (
                Some(Value::Integer(id)),
                Some(Value::Text(name)),
                Some(Value::Text(table_name)),
                Some(Value::Text(timing)),
                Some(Value::Text(event)),
                Some(Value::Boolean(for_each_row)),
                Some(Value::Text(language)),
                Some(Value::Text(code)),
            ) = (
                row.get(0),
                row.get(2),
                row.get(3),
                row.get(4),
                row.get(5),
                row.get(6),
                row.get(7),
                row.get(8),
            ) {
                let schema_val = row.get(1).and_then(|v| match v {
                    Value::Text(s) => Some(s.to_string()),
                    _ => None,
                });

                triggers.push(crate::storage::triggers::StoredTrigger {
                    id: *id,
                    schema: schema_val,
                    name: name.to_string(),
                    table_name: table_name.to_string(),
                    timing: timing.to_string(),
                    event: event.to_string(),
                    for_each_row: *for_each_row,
                    language: language.to_string(),
                    code: code.to_string(),
                });
            }
        }

        self.trigger_registry.load_triggers(triggers);
        Ok(())
    }
