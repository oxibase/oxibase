import re

with open("src/executor/ddl.rs", "r") as f:
    content = f.read()

METHODS = """
    pub(crate) fn execute_create_schedule(
        &self,
        stmt: &CreateScheduleStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        self.ensure_cron_tables_exist()?;

        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(crate::storage::jobs::SYS_CRON)?;

        // Ensure schedule is valid
        if let Err(e) = stmt.cron_expr.parse::<cron::Schedule>() {
            return Err(Error::internal(format!("Invalid CRON expression: {}", e)));
        }

        let values = vec![
            crate::core::value::Value::Null(DataType::Integer), // ID (auto increment)
            crate::core::value::Value::text(stmt.name.to_uppercase()),
            crate::core::value::Value::text(stmt.cron_expr.clone()),
            crate::core::value::Value::text(stmt.command.clone()),
            crate::core::value::Value::Boolean(true), // active
        ];

        let row = Row::from_values(values);
        table.insert(row)?;
        tx.commit()?;

        Ok(Box::new(ExecResult::new(1, 0)))
    }

    pub(crate) fn execute_alter_schedule(
        &self,
        stmt: &AlterScheduleStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        use crate::executor::expression::RowFilter;

        self.ensure_cron_tables_exist()?;

        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(crate::storage::jobs::SYS_CRON)?;

        let name_upper = stmt.name.to_uppercase();
        let where_expr = crate::parser::ast::Expression::Infix(crate::parser::ast::InfixExpression::new(
            crate::parser::token::Token::new(crate::parser::token::TokenType::Operator, "=", crate::parser::token::Position::default()),
            Box::new(crate::parser::ast::Expression::Identifier(crate::parser::ast::Identifier {
                token: crate::parser::token::Token::new(crate::parser::token::TokenType::Identifier, "name", crate::parser::token::Position::default()),
                value: "name".to_string(),
                value_lower: "name".to_string(),
            })),
            "=".to_string(),
            Box::new(crate::parser::ast::Expression::StringLiteral(crate::parser::ast::StringLiteral {
                token: crate::parser::token::Token::new(crate::parser::token::TokenType::String, name_upper.clone(), crate::parser::token::Position::default()),
                value: name_upper.clone(),
                type_hint: None,
            })),
        ));

        let schema = table.schema();
        let col_names: Vec<String> = schema.column_names().iter().map(|s| s.to_string()).collect();
        let row_filter = RowFilter::new(&where_expr, &col_names)?;

        let mut scanner = table.scan(&[], None)?;
        let mut id_to_update = None;

        while scanner.next() {
            let row = scanner.row();
            if row_filter.matches(row) {
                if let Some(crate::core::value::Value::Integer(id)) = row.get(0) {
                    id_to_update = Some(*id);
                    break;
                }
            }
        }

        if let Some(id) = id_to_update {
            let pk_expr = crate::storage::expression::ComparisonExpr::new(
                "id".to_string(),
                crate::core::Operator::Eq,
                crate::core::value::Value::Integer(id),
            );
            
            let mut setter = |mut row: Row| -> Result<(Row, bool)> {
                let _ = row.set(4, crate::core::value::Value::Boolean(stmt.active));
                Ok((row, true))
            };
            
            table.update(Some(&pk_expr), &mut setter)?;
            tx.commit()?;
            Ok(Box::new(ExecResult::new(0, 1)))
        } else {
            return Err(Error::internal(format!("Schedule not found: {}", name_upper)));
        }
    }

    pub(crate) fn execute_drop_schedule(
        &self,
        stmt: &DropScheduleStatement,
        _ctx: &ExecutionContext,
    ) -> Result<Box<dyn QueryResult>> {
        use crate::executor::expression::RowFilter;
        
        self.ensure_cron_tables_exist()?;

        let mut tx = self.engine.begin_transaction()?;
        let mut table = tx.get_table(crate::storage::jobs::SYS_CRON)?;

        let name_upper = stmt.name.to_uppercase();
        let where_expr = crate::parser::ast::Expression::Infix(crate::parser::ast::InfixExpression::new(
            crate::parser::token::Token::new(crate::parser::token::TokenType::Operator, "=", crate::parser::token::Position::default()),
            Box::new(crate::parser::ast::Expression::Identifier(crate::parser::ast::Identifier {
                token: crate::parser::token::Token::new(crate::parser::token::TokenType::Identifier, "name", crate::parser::token::Position::default()),
                value: "name".to_string(),
                value_lower: "name".to_string(),
            })),
            "=".to_string(),
            Box::new(crate::parser::ast::Expression::StringLiteral(crate::parser::ast::StringLiteral {
                token: crate::parser::token::Token::new(crate::parser::token::TokenType::String, name_upper.clone(), crate::parser::token::Position::default()),
                value: name_upper.clone(),
                type_hint: None,
            })),
        ));

        let schema = table.schema();
        let col_names: Vec<String> = schema.column_names().iter().map(|s| s.to_string()).collect();
        let row_filter = RowFilter::new(&where_expr, &col_names)?;

        let mut scanner = table.scan(&[], None)?;
        let mut id_to_delete = None;

        while scanner.next() {
            let row = scanner.row();
            if row_filter.matches(row) {
                if let Some(crate::core::value::Value::Integer(id)) = row.get(0) {
                    id_to_delete = Some(*id);
                    break;
                }
            }
        }

        let affected = if let Some(id) = id_to_delete {
            let mut pk_expr = crate::storage::expression::ComparisonExpr::new(
                "id".to_string(),
                crate::core::Operator::Eq,
                crate::core::value::Value::Integer(id),
            );
            pk_expr.prepare_for_schema(&schema);
            table.delete(Some(&pk_expr))?
        } else {
            0
        };
        
        tx.commit()?;

        Ok(Box::new(ExecResult::new(0, affected as i64)))
    }

"""

content = content.replace("    pub(crate) fn execute_create_trigger(", METHODS + "    pub(crate) fn execute_create_trigger(")

with open("src/executor/ddl.rs", "w") as f:
    f.write(content)
