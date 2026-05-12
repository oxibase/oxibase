
    pub(crate) fn ensure_triggers_table_exists(&self) -> Result<()> {
        let tx = self.engine.begin_transaction()?;
        let tables = tx.list_tables()?;
        let has_triggers_table = tables
            .iter()
            .any(|t| t.eq_ignore_ascii_case(crate::storage::triggers::SYS_TRIGGERS));
        drop(tx);

        if !has_triggers_table {
            self.execute_functions_sql(crate::storage::triggers::CREATE_TRIGGERS_SQL)?;
        }
        Ok(())
    }

    fn trigger_exists(&self, trigger_name: &str) -> Result<bool> {
        let tx = self.engine.begin_transaction()?;
        let tables = tx.list_tables()?;
        if !tables.iter().any(|t| t.eq_ignore_ascii_case(crate::storage::triggers::SYS_TRIGGERS)) {
            return Ok(false);
        }

        let table = tx.get_table(crate::storage::triggers::SYS_TRIGGERS)?;
        let mut scanner = table.scan(&[], None)?;
        
        while scanner.next() {
            let row = scanner.row();
            if let Some(Value::Text(name)) = row.get(2) {
                if name.eq_ignore_ascii_case(trigger_name) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    fn insert_trigger(&self, trigger: &crate::storage::triggers::StoredTrigger) -> Result<()> {
        let (mut tx, mut table, auto_commit) = self.start_transaction_for_dml(crate::storage::triggers::SYS_TRIGGERS)?;

        let mut row_values = vec![
            Value::Null, // id auto increment
            trigger.schema.as_ref().map(|s| Value::text(s.clone())).unwrap_or(Value::Null),
            Value::text(trigger.name.clone()),
            Value::text(trigger.table_name.clone()),
            Value::text(trigger.timing.clone()),
            Value::text(trigger.event.clone()),
            Value::Boolean(trigger.for_each_row),
            Value::text(trigger.language.clone()),
            Value::text(trigger.code.clone()),
        ];

        table.insert(&mut row_values)?;

        if auto_commit {
            if let Some(tx) = tx {
                tx.commit()?;
            }
        }
        Ok(())
    }

    fn delete_trigger(&self, trigger_name: &str) -> Result<bool> {
        let (mut tx, mut table, auto_commit) = self.start_transaction_for_dml(crate::storage::triggers::SYS_TRIGGERS)?;

        let mut ids_to_delete = Vec::new();
        let mut scanner = table.scan(&[], None)?;
        while scanner.next() {
            let row = scanner.row();
            if let (Some(Value::Integer(id)), Some(Value::Text(name))) = (row.get(0), row.get(2)) {
                if name.eq_ignore_ascii_case(trigger_name) {
                    ids_to_delete.push(*id);
                }
            }
        }

        let mut deleted = false;
        for id in ids_to_delete {
            table.delete(&[Value::Integer(id)])?;
            deleted = true;
        }

        if auto_commit {
            if let Some(tx) = tx {
                tx.commit()?;
            }
        }
        Ok(deleted)
    }

    pub(crate) fn execute_create_trigger(
        &self,
        stmt: &crate::parser::ast::CreateTriggerStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        self.ensure_triggers_table_exists()?;

        let trigger_name_upper = stmt.trigger_name.value.to_uppercase();
        let exists = self.trigger_exists(&trigger_name_upper)?;

        if exists && !stmt.if_not_exists {
            return Err(Error::internal(format!("Trigger {} already exists", trigger_name_upper)));
        } else if exists && stmt.if_not_exists {
            return Ok(Box::new(ExecResult::new(0)));
        }

        if !self.function_registry.is_language_supported(&stmt.language) {
            return Err(Error::internal(format!(
                "Unsupported language for trigger: {}",
                stmt.language
            )));
        }

        let stored_trigger = crate::storage::triggers::StoredTrigger {
            id: 0,
            schema: None,
            name: trigger_name_upper.clone(),
            table_name: stmt.table_name.value().to_uppercase(),
            timing: stmt.timing.to_string(),
            event: stmt.event.to_string(),
            for_each_row: stmt.for_each_row,
            language: stmt.language.clone(),
            code: stmt.body.clone(),
        };

        self.insert_trigger(&stored_trigger)?;

        Ok(Box::new(ExecResult::new(1)))
    }

    pub(crate) fn execute_drop_trigger(
        &self,
        stmt: &crate::parser::ast::DropTriggerStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        self.ensure_triggers_table_exists()?;

        let trigger_name_upper = stmt.trigger_name.value.to_uppercase();
        let deleted = self.delete_trigger(&trigger_name_upper)?;

        if !deleted && !stmt.if_exists {
            return Err(Error::internal(format!("Trigger {} does not exist", trigger_name_upper)));
        }

        Ok(Box::new(ExecResult::new(if deleted { 1 } else { 0 })))
    }
