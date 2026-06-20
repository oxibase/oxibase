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

//! Script static analysis and dependency extraction orchestrator

use crate::api::database::RelatedObject;
use crate::core::{Error, Result};
use crate::parser::parse_sql;
use crate::parser::visitor::{DependencyExtractor, Visitor};
use std::collections::HashSet;

/// Statically analyze a procedural script to detect referenced database objects.
pub fn analyze_script(script: &str, backend: &str) -> Result<Vec<RelatedObject>> {
    let mut tables = HashSet::new();
    let mut procedures = HashSet::new();
    let mut functions = HashSet::new();
    let mut is_dynamic = false;

    match backend.to_lowercase().as_str() {
        "rhai" => {
            let (sql_queries, dyn_flag) = extract_sql_from_rhai(script)?;
            is_dynamic = dyn_flag;
            for sql in sql_queries {
                if let Ok(statements) = parse_sql(&sql) {
                    let mut extractor = DependencyExtractor::new();
                    for stmt in &statements {
                        extractor.visit_statement(stmt);
                    }
                    tables.extend(extractor.tables);
                    procedures.extend(extractor.procedures);
                    functions.extend(extractor.functions);
                }
            }
        }
        "python" | "py" => {
            #[cfg(feature = "python")]
            {
                let (sql_queries, dyn_flag) = extract_sql_from_python(script)?;
                is_dynamic = dyn_flag;
                for sql in sql_queries {
                    if let Ok(statements) = parse_sql(&sql) {
                        let mut extractor = DependencyExtractor::new();
                        for stmt in &statements {
                            extractor.visit_statement(stmt);
                        }
                        tables.extend(extractor.tables);
                        procedures.extend(extractor.procedures);
                        functions.extend(extractor.functions);
                    }
                }
            }
            #[cfg(not(feature = "python"))]
            {
                return Err(Error::internal("Python scripting backend is not enabled"));
            }
        }
        "plsql" | "sql" => {
            // For PL/SQL or raw SQL, we parse the whole script as SQL directly.
            if let Ok(statements) = parse_sql(script) {
                let mut extractor = DependencyExtractor::new();
                for stmt in &statements {
                    extractor.visit_statement(stmt);
                }
                tables.extend(extractor.tables);
                procedures.extend(extractor.procedures);
                functions.extend(extractor.functions);
            }
        }
        _ => {
            return Err(Error::internal(format!(
                "Unsupported scripting backend: {}",
                backend
            )));
        }
    }

    let mut related_objects = Vec::new();

    for table in tables {
        related_objects.push(RelatedObject {
            object_type: "Table".to_string(),
            name: table,
        });
    }
    for proc in procedures {
        related_objects.push(RelatedObject {
            object_type: "Procedure".to_string(),
            name: proc,
        });
    }
    for func in functions {
        related_objects.push(RelatedObject {
            object_type: "Function".to_string(),
            name: func,
        });
    }

    if is_dynamic {
        related_objects.push(RelatedObject {
            object_type: "Dynamic".to_string(),
            name: "Dynamic".to_string(),
        });
    }

    // Sort related objects by type, then name for determinism
    related_objects.sort_by(|a, b| {
        a.object_type
            .cmp(&b.object_type)
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(related_objects)
}

fn extract_sql_from_rhai(script: &str) -> Result<(Vec<String>, bool)> {
    let engine = rhai::Engine::new();
    let ast = engine
        .compile(script)
        .map_err(|e| Error::internal(format!("Rhai compilation error: {}", e)))?;

    let mut sql_queries = Vec::new();
    let mut is_dynamic = false;

    // Walk main script statements
    for stmt in ast.statements() {
        walk_rhai_stmt(stmt, &mut sql_queries, &mut is_dynamic);
    }

    Ok((sql_queries, is_dynamic))
}

fn walk_rhai_stmt(stmt: &rhai::Stmt, sql_queries: &mut Vec<String>, is_dynamic: &mut bool) {
    match stmt {
        rhai::Stmt::Expr(expr) => {
            walk_rhai_expr(expr, sql_queries, is_dynamic);
        }
        rhai::Stmt::FnCall(fc, _) => {
            walk_rhai_fn_call(fc, sql_queries, is_dynamic);
        }
        rhai::Stmt::Block(block) => {
            for s in block.as_ref().iter() {
                walk_rhai_stmt(s, sql_queries, is_dynamic);
            }
        }
        rhai::Stmt::If(fc, _) => {
            walk_rhai_expr(&fc.expr, sql_queries, is_dynamic);
            for s in fc.body.as_ref().iter() {
                walk_rhai_stmt(s, sql_queries, is_dynamic);
            }
            for s in fc.branch.as_ref().iter() {
                walk_rhai_stmt(s, sql_queries, is_dynamic);
            }
        }
        rhai::Stmt::While(fc, _) => {
            walk_rhai_expr(&fc.expr, sql_queries, is_dynamic);
            for s in fc.body.as_ref().iter() {
                walk_rhai_stmt(s, sql_queries, is_dynamic);
            }
        }
        rhai::Stmt::Do(fc, _, _) => {
            for s in fc.body.as_ref().iter() {
                walk_rhai_stmt(s, sql_queries, is_dynamic);
            }
            walk_rhai_expr(&fc.expr, sql_queries, is_dynamic);
        }
        rhai::Stmt::For(args, _) => {
            let fc = &args.2;
            walk_rhai_expr(&fc.expr, sql_queries, is_dynamic);
            for s in fc.body.as_ref().iter() {
                walk_rhai_stmt(s, sql_queries, is_dynamic);
            }
        }
        rhai::Stmt::Var(args, _, _) => {
            walk_rhai_expr(&args.1, sql_queries, is_dynamic);
        }
        rhai::Stmt::Assignment(args) => {
            let binary_expr = &args.1;
            walk_rhai_expr(&binary_expr.lhs, sql_queries, is_dynamic);
            walk_rhai_expr(&binary_expr.rhs, sql_queries, is_dynamic);
        }
        _ => {}
    }
}

fn walk_rhai_fn_call(
    fn_call: &rhai::FnCallExpr,
    sql_queries: &mut Vec<String>,
    is_dynamic: &mut bool,
) {
    let name = fn_call.name.as_str();
    if name == "oxibase::execute"
        || name == "execute"
        || name == "oxibase::query"
        || name == "query"
        || name == "oxibase::call"
        || name == "call"
    {
        if let Some(rhai::Expr::StringConstant(sql, _)) = fn_call.args.first() {
            sql_queries.push(sql.to_string());
        } else {
            *is_dynamic = true;
        }
    }
    // Walk function arguments recursively
    for arg in fn_call.args.iter() {
        walk_rhai_expr(arg, sql_queries, is_dynamic);
    }
}

fn walk_rhai_expr(expr: &rhai::Expr, sql_queries: &mut Vec<String>, is_dynamic: &mut bool) {
    match expr {
        rhai::Expr::FnCall(fn_call, _) => {
            walk_rhai_fn_call(fn_call, sql_queries, is_dynamic);
        }
        rhai::Expr::And(args, _) => {
            for arg in args.iter() {
                walk_rhai_expr(arg, sql_queries, is_dynamic);
            }
        }
        rhai::Expr::Or(args, _) => {
            for arg in args.iter() {
                walk_rhai_expr(arg, sql_queries, is_dynamic);
            }
        }
        rhai::Expr::Dot(args, _, _) => {
            walk_rhai_expr(&args.lhs, sql_queries, is_dynamic);
            walk_rhai_expr(&args.rhs, sql_queries, is_dynamic);
        }
        rhai::Expr::Index(args, _, _) => {
            walk_rhai_expr(&args.lhs, sql_queries, is_dynamic);
            walk_rhai_expr(&args.rhs, sql_queries, is_dynamic);
        }
        rhai::Expr::Array(exprs, _) => {
            for e in exprs.iter() {
                walk_rhai_expr(e, sql_queries, is_dynamic);
            }
        }
        rhai::Expr::Map(args, _) => {
            for (_, e) in args.0.iter() {
                walk_rhai_expr(e, sql_queries, is_dynamic);
            }
        }
        _ => {}
    }
}

#[cfg(feature = "python")]
fn extract_sql_from_python(script: &str) -> Result<(Vec<String>, bool)> {
    let options = rustpython_vm::compiler::parser::ParseOptions::from(
        rustpython_vm::compiler::parser::Mode::Module,
    );

    let parsed = rustpython_vm::compiler::parser::parse(script, options)
        .map_err(|e| Error::internal(format!("Python parsing error: {}", e)))?;

    let mut sql_queries = Vec::new();
    let mut is_dynamic = false;

    if let rustpython_vm::compiler::ast::Mod::Module(module) = parsed.syntax() {
        for stmt in &module.body {
            walk_python_stmt(stmt, &mut sql_queries, &mut is_dynamic);
        }
    }

    Ok((sql_queries, is_dynamic))
}

#[cfg(feature = "python")]
fn walk_python_stmt(
    stmt: &rustpython_vm::compiler::ast::Stmt,
    sql_queries: &mut Vec<String>,
    is_dynamic: &mut bool,
) {
    use rustpython_vm::compiler::ast::Stmt;
    match stmt {
        Stmt::Expr(expr_stmt) => {
            walk_python_expr(&expr_stmt.value, sql_queries, is_dynamic);
        }
        Stmt::FunctionDef(func_def) => {
            for s in &func_def.body {
                walk_python_stmt(s, sql_queries, is_dynamic);
            }
        }
        Stmt::ClassDef(class_def) => {
            for s in &class_def.body {
                walk_python_stmt(s, sql_queries, is_dynamic);
            }
        }
        Stmt::If(if_stmt) => {
            for s in &if_stmt.body {
                walk_python_stmt(s, sql_queries, is_dynamic);
            }
            for clause in &if_stmt.elif_else_clauses {
                for s in &clause.body {
                    walk_python_stmt(s, sql_queries, is_dynamic);
                }
            }
        }
        Stmt::While(while_stmt) => {
            for s in &while_stmt.body {
                walk_python_stmt(s, sql_queries, is_dynamic);
            }
            for s in &while_stmt.orelse {
                walk_python_stmt(s, sql_queries, is_dynamic);
            }
        }
        Stmt::For(for_stmt) => {
            for s in &for_stmt.body {
                walk_python_stmt(s, sql_queries, is_dynamic);
            }
            for s in &for_stmt.orelse {
                walk_python_stmt(s, sql_queries, is_dynamic);
            }
        }
        Stmt::Try(try_stmt) => {
            for s in &try_stmt.body {
                walk_python_stmt(s, sql_queries, is_dynamic);
            }
            for handler in &try_stmt.handlers {
                match handler {
                    rustpython_vm::compiler::ast::ExceptHandler::ExceptHandler(h) => {
                        for s in &h.body {
                            walk_python_stmt(s, sql_queries, is_dynamic);
                        }
                    }
                }
            }
            for s in &try_stmt.orelse {
                walk_python_stmt(s, sql_queries, is_dynamic);
            }
            for s in &try_stmt.finalbody {
                walk_python_stmt(s, sql_queries, is_dynamic);
            }
        }
        Stmt::Assign(assign_stmt) => {
            walk_python_expr(&assign_stmt.value, sql_queries, is_dynamic);
        }
        Stmt::AugAssign(aug_stmt) => {
            walk_python_expr(&aug_stmt.value, sql_queries, is_dynamic);
        }
        Stmt::Return(ret_stmt) => {
            if let Some(expr) = &ret_stmt.value {
                walk_python_expr(expr, sql_queries, is_dynamic);
            }
        }
        _ => {}
    }
}

#[cfg(feature = "python")]
fn walk_python_expr(
    expr: &rustpython_vm::compiler::ast::Expr,
    sql_queries: &mut Vec<String>,
    is_dynamic: &mut bool,
) {
    use rustpython_vm::compiler::ast::Expr;
    match expr {
        Expr::Call(call_expr) => {
            let is_target_call = match &*call_expr.func {
                Expr::Attribute(attr_expr) => {
                    if let Expr::Name(name_expr) = &*attr_expr.value {
                        let name = name_expr.id.as_str();
                        let attr = attr_expr.attr.as_str();
                        name == "oxibase"
                            && (attr == "execute" || attr == "query" || attr == "call")
                    } else {
                        false
                    }
                }
                Expr::Name(name_expr) => {
                    let name = name_expr.id.as_str();
                    name == "execute" || name == "query" || name == "call"
                }
                _ => false,
            };

            if is_target_call {
                if let Some(Expr::StringLiteral(str_lit)) = call_expr.arguments.args.first() {
                    sql_queries.push(str_lit.value.to_string());
                } else {
                    *is_dynamic = true;
                }
            }

            for arg in &call_expr.arguments.args {
                walk_python_expr(arg, sql_queries, is_dynamic);
            }
        }
        Expr::BoolOp(bool_op) => {
            for value in &bool_op.values {
                walk_python_expr(value, sql_queries, is_dynamic);
            }
        }
        Expr::BinOp(bin_op) => {
            walk_python_expr(&bin_op.left, sql_queries, is_dynamic);
            walk_python_expr(&bin_op.right, sql_queries, is_dynamic);
        }
        Expr::UnaryOp(unary_op) => {
            walk_python_expr(&unary_op.operand, sql_queries, is_dynamic);
        }
        Expr::Lambda(lambda) => {
            walk_python_expr(&lambda.body, sql_queries, is_dynamic);
        }
        Expr::Dict(dict) => {
            for item in &dict.items {
                walk_python_expr(&item.value, sql_queries, is_dynamic);
            }
        }
        Expr::Set(set) => {
            for el in &set.elts {
                walk_python_expr(el, sql_queries, is_dynamic);
            }
        }
        Expr::List(list) => {
            for el in &list.elts {
                walk_python_expr(el, sql_queries, is_dynamic);
            }
        }
        Expr::Tuple(tuple) => {
            for el in &tuple.elts {
                walk_python_expr(el, sql_queries, is_dynamic);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_rhai_script() {
        let script = r#"
            oxibase::execute("INSERT INTO orders VALUES (1, 'Alice')");
            let x = 42;
            if x > 10 {
                oxibase::query("SELECT * FROM users JOIN products ON users.id = products.id");
            }
            oxibase::execute("CALL my_procedure()");
        "#;

        let results = analyze_script(script, "rhai").unwrap();

        let tables: Vec<&str> = results
            .iter()
            .filter(|o| o.object_type == "Table")
            .map(|o| o.name.as_str())
            .collect();
        let procedures: Vec<&str> = results
            .iter()
            .filter(|o| o.object_type == "Procedure")
            .map(|o| o.name.as_str())
            .collect();

        assert!(tables.contains(&"orders"));
        assert!(tables.contains(&"users"));
        assert!(tables.contains(&"products"));
        assert!(procedures.contains(&"my_procedure"));
    }

    #[test]
    fn test_analyze_rhai_dynamic_query() {
        let script = r#"
            let tbl = "users";
            oxibase::execute("SELECT * FROM " + tbl);
        "#;

        let results = analyze_script(script, "rhai").unwrap();
        assert!(results.iter().any(|o| o.object_type == "Dynamic"));
    }

    #[test]
    #[cfg(feature = "python")]
    fn test_analyze_python_script() {
        let script = r#"
import oxibase
oxibase.execute("UPDATE products SET stock = stock - 1 WHERE id = 10")
if True:
    oxibase.query("SELECT * FROM customers")
"#;

        let results = analyze_script(script, "python").unwrap();

        let tables: Vec<&str> = results
            .iter()
            .filter(|o| o.object_type == "Table")
            .map(|o| o.name.as_str())
            .collect();

        assert!(tables.contains(&"products"));
        assert!(tables.contains(&"customers"));
    }

    #[test]
    #[cfg(feature = "python")]
    fn test_analyze_python_dynamic_query() {
        let script = r#"
import oxibase
sql = get_dynamic_query()
oxibase.execute(sql)
"#;

        let results = analyze_script(script, "python").unwrap();
        assert!(results.iter().any(|o| o.object_type == "Dynamic"));
    }

    #[test]
    fn test_analyze_plsql_script() {
        let script = r#"
            SELECT * FROM employees;
            CALL notify_admin();
        "#;

        let results = analyze_script(script, "plsql").unwrap();

        let tables: Vec<&str> = results
            .iter()
            .filter(|o| o.object_type == "Table")
            .map(|o| o.name.as_str())
            .collect();
        let procedures: Vec<&str> = results
            .iter()
            .filter(|o| o.object_type == "Procedure")
            .map(|o| o.name.as_str())
            .collect();

        assert!(tables.contains(&"employees"));
        assert!(procedures.contains(&"notify_admin"));
    }
}
