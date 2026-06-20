// Copyright 2026 Oxibase Contributors
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

//! SQL AST Visitor and Dependency Extractor

use crate::parser::ast::*;
use std::collections::HashSet;

/// Visitor pattern for traversing the SQL AST.
pub trait Visitor {
    /// Visit a SQL Statement.
    fn visit_statement(&mut self, stmt: &Statement) {
        walk_statement(self, stmt);
    }

    /// Visit a SQL Expression.
    fn visit_expression(&mut self, expr: &Expression) {
        walk_expression(self, expr);
    }
}

/// Helper function to traverse a SQL Statement recursively.
pub fn walk_statement<V: Visitor + ?Sized>(visitor: &mut V, stmt: &Statement) {
    match stmt {
        Statement::Select(select) => {
            for col in &select.columns {
                visitor.visit_expression(col);
            }
            if let Some(table_expr) = &select.table_expr {
                visitor.visit_expression(table_expr);
            }
            if let Some(where_clause) = &select.where_clause {
                visitor.visit_expression(where_clause);
            }
            if let Some(having) = &select.having {
                visitor.visit_expression(having);
            }
            for order in &select.order_by {
                visitor.visit_expression(&order.expression);
            }
            if let Some(limit) = &select.limit {
                visitor.visit_expression(limit);
            }
            if let Some(offset) = &select.offset {
                visitor.visit_expression(offset);
            }
            for set_op in &select.set_operations {
                visitor.visit_statement(&Statement::Select(*set_op.right.clone()));
            }
        }
        Statement::Insert(insert) => {
            for row in &insert.values {
                for val in row {
                    visitor.visit_expression(val);
                }
            }
            if let Some(select) = &insert.select {
                visitor.visit_statement(&Statement::Select(*select.clone()));
            }
        }
        Statement::Update(update) => {
            for val in update.updates.values() {
                visitor.visit_expression(val);
            }
            if let Some(where_clause) = &update.where_clause {
                visitor.visit_expression(where_clause);
            }
        }
        Statement::Delete(delete) => {
            if let Some(where_clause) = &delete.where_clause {
                visitor.visit_expression(where_clause);
            }
        }
        Statement::Call(call) => {
            for arg in &call.arguments {
                visitor.visit_expression(arg);
            }
        }
        Statement::Expression(expr_stmt) => {
            visitor.visit_expression(&expr_stmt.expression);
        }
        Statement::Explain(explain) => {
            visitor.visit_statement(&explain.statement);
        }
        Statement::CreateView(create_view) => {
            visitor.visit_statement(&Statement::Select(*create_view.query.clone()));
        }
        _ => {}
    }
}

/// Helper function to traverse a SQL Expression recursively.
pub fn walk_expression<V: Visitor + ?Sized>(visitor: &mut V, expr: &Expression) {
    match expr {
        Expression::Prefix(prefix) => {
            visitor.visit_expression(&prefix.right);
        }
        Expression::Infix(infix) => {
            visitor.visit_expression(&infix.left);
            visitor.visit_expression(&infix.right);
        }
        Expression::List(list) => {
            for e in &list.elements {
                visitor.visit_expression(e);
            }
        }
        Expression::Distinct(distinct) => {
            visitor.visit_expression(&distinct.expr);
        }
        Expression::Exists(exists) => {
            visitor.visit_statement(&Statement::Select(*exists.subquery.clone()));
        }
        Expression::AllAny(all_any) => {
            visitor.visit_expression(&all_any.left);
            visitor.visit_statement(&Statement::Select(*all_any.subquery.clone()));
        }
        Expression::In(in_expr) => {
            visitor.visit_expression(&in_expr.left);
            visitor.visit_expression(&in_expr.right);
        }
        Expression::InHashSet(in_hs) => {
            visitor.visit_expression(&in_hs.column);
        }
        Expression::Between(between) => {
            visitor.visit_expression(&between.expr);
            visitor.visit_expression(&between.lower);
            visitor.visit_expression(&between.upper);
        }
        Expression::Like(like) => {
            visitor.visit_expression(&like.left);
            visitor.visit_expression(&like.pattern);
            if let Some(escape) = &like.escape {
                visitor.visit_expression(escape);
            }
        }
        Expression::ScalarSubquery(subquery) => {
            visitor.visit_statement(&Statement::Select(*subquery.subquery.clone()));
        }
        Expression::ExpressionList(list) => {
            for e in &list.expressions {
                visitor.visit_expression(e);
            }
        }
        Expression::Case(case) => {
            if let Some(val) = &case.value {
                visitor.visit_expression(val);
            }
            for when in &case.when_clauses {
                visitor.visit_expression(&when.condition);
                visitor.visit_expression(&when.then_result);
            }
            if let Some(else_val) = &case.else_value {
                visitor.visit_expression(else_val);
            }
        }
        Expression::Cast(cast) => {
            visitor.visit_expression(&cast.expr);
        }
        Expression::FunctionCall(func) => {
            for arg in &func.arguments {
                visitor.visit_expression(arg);
            }
            if let Some(filter) = &func.filter {
                visitor.visit_expression(filter);
            }
        }
        Expression::Aliased(aliased) => {
            visitor.visit_expression(&aliased.expression);
        }
        Expression::Window(window) => {
            visitor.visit_expression(&Expression::FunctionCall(*window.function.clone()));
            for partition in &window.partition_by {
                visitor.visit_expression(partition);
            }
            for order in &window.order_by {
                visitor.visit_expression(&order.expression);
            }
        }
        Expression::JoinSource(join) => {
            visitor.visit_expression(&join.left);
            visitor.visit_expression(&join.right);
            if let Some(cond) = &join.condition {
                visitor.visit_expression(cond);
            }
        }
        Expression::SubquerySource(subquery) => {
            visitor.visit_statement(&Statement::Select(*subquery.subquery.clone()));
        }
        Expression::ValuesSource(values) => {
            for row in &values.rows {
                for expr in row {
                    visitor.visit_expression(expr);
                }
            }
        }
        Expression::FunctionTableSource(func) => {
            for arg in &func.arguments {
                visitor.visit_expression(arg);
            }
        }
        _ => {}
    }
}

/// Visitor implementation to extract all tables, procedures, and functions referenced in the SQL AST.
#[derive(Debug, Default)]
pub struct DependencyExtractor {
    /// Extracted table names
    pub tables: HashSet<String>,
    /// Extracted procedure names
    pub procedures: HashSet<String>,
    /// Extracted function names
    pub functions: HashSet<String>,
}

impl DependencyExtractor {
    /// Create a new empty DependencyExtractor
    pub fn new() -> Self {
        Self {
            tables: HashSet::new(),
            procedures: HashSet::new(),
            functions: HashSet::new(),
        }
    }
}

impl Visitor for DependencyExtractor {
    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Insert(insert) => {
                self.tables.insert(insert.table_name.value());
            }
            Statement::Update(update) => {
                self.tables.insert(update.table_name.value());
            }
            Statement::Delete(delete) => {
                self.tables.insert(delete.table_name.value());
            }
            Statement::Truncate(truncate) => {
                self.tables.insert(truncate.table_name.value.clone());
            }
            Statement::Call(call) => {
                self.procedures.insert(call.procedure_name.value());
            }
            _ => {}
        }
        walk_statement(self, stmt);
    }

    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::TableSource(ts) => {
                self.tables.insert(ts.name.value());
            }
            Expression::FunctionTableSource(fts) => {
                self.functions.insert(fts.function.value.clone());
            }
            Expression::FunctionCall(fc) => {
                self.functions.insert(fc.function.clone());
            }
            _ => {}
        }
        walk_expression(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_sql;

    #[test]
    fn test_extract_select_tables() {
        let sql = "SELECT * FROM users JOIN orders ON users.id = orders.user_id WHERE id = 1";
        let statements = parse_sql(sql).unwrap();
        let mut extractor = DependencyExtractor::new();
        for stmt in &statements {
            extractor.visit_statement(stmt);
        }
        assert!(extractor.tables.contains("users"));
        assert!(extractor.tables.contains("orders"));
        assert_eq!(extractor.tables.len(), 2);
    }

    #[test]
    fn test_extract_insert_update_delete_truncate() {
        let ins = parse_sql("INSERT INTO t1 VALUES (1)").unwrap();
        let upd = parse_sql("UPDATE t2 SET val = 2").unwrap();
        let del = parse_sql("DELETE FROM t3").unwrap();
        let tru = parse_sql("TRUNCATE TABLE t4").unwrap();

        let mut extractor = DependencyExtractor::new();
        for s in ins.iter().chain(&upd).chain(&del).chain(&tru) {
            extractor.visit_statement(s);
        }

        assert!(extractor.tables.contains("t1"));
        assert!(extractor.tables.contains("t2"));
        assert!(extractor.tables.contains("t3"));
        assert!(extractor.tables.contains("t4"));
        assert_eq!(extractor.tables.len(), 4);
    }

    #[test]
    fn test_extract_procedure_call() {
        let sql = "CALL pizza_demo.simulate_random_order()";
        let statements = parse_sql(sql).unwrap();
        let mut extractor = DependencyExtractor::new();
        for stmt in &statements {
            extractor.visit_statement(stmt);
        }
        assert!(extractor
            .procedures
            .contains("pizza_demo.simulate_random_order"));
        assert_eq!(extractor.procedures.len(), 1);
    }

    #[test]
    fn test_extract_function_call_and_tvf() {
        let sql = "SELECT ABS(val) FROM generate_series(1, 10)";
        let statements = parse_sql(sql).unwrap();
        let mut extractor = DependencyExtractor::new();
        for stmt in &statements {
            extractor.visit_statement(stmt);
        }
        assert!(extractor.functions.contains("ABS"));
        assert!(extractor.functions.contains("generate_series"));
        assert_eq!(extractor.functions.len(), 2);
    }
}
