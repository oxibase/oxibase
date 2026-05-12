    fn delete_table_triggers(&self, table_name: &str) -> Result<()> {
        let (mut tx, mut table, auto_commit) = self.start_transaction_for_dml(crate::storage::triggers::SYS_TRIGGERS)?;
        let mut ids_to_delete = Vec::new();
        let mut scanner = table.scan(&[], None)?;
        
        while scanner.next() {
            let row = scanner.row();
            if let (Some(Value::Integer(id)), Some(Value::Text(target))) = (row.get(0), row.get(3)) {
                if target.eq_ignore_ascii_case(table_name) {
                    ids_to_delete.push(*id);
                }
            }
        }
        
        for id in ids_to_delete {
            let mut pk_expr = crate::storage::expression::ComparisonExpr::new("id", crate::core::Operator::Eq, Value::Integer(id));
            pk_expr.prepare_for_schema(table.schema());
            let _ = table.delete(Some(&pk_expr))?;
        }
        
        if auto_commit {
            if let Some(mut tx) = tx {
                tx.commit()?;
            }
        }
        
        self.trigger_registry.remove_table_triggers(table_name);
        Ok(())
    }
