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

use super::ast::{BlockStatement, PlSqlStatement};
use super::env::Environment;
use crate::core::{Error, Result, Value};
use crate::functions::FunctionRegistry;
use std::sync::Arc;

pub struct PlSqlInterpreter<'a> {
    pub(crate) _function_registry: Arc<FunctionRegistry>,
    runner: Option<&'a dyn crate::functions::backends::SqlRunner>,
}

impl<'a> PlSqlInterpreter<'a> {
    pub fn new(
        function_registry: Arc<FunctionRegistry>,
        runner: Option<&'a dyn crate::functions::backends::SqlRunner>,
    ) -> Self {
        Self {
            _function_registry: function_registry,
            runner,
        }
    }

    pub fn execute(&self, block: &BlockStatement, env: &mut Environment) -> Result<()> {
        for stmt in &block.statements {
            match self.execute_statement(stmt, env) {
                Ok(true) => return Ok(()), // RETURN statement executed
                Ok(false) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn eval_expr(&self, expr: &crate::parser::ast::Expression, env: &Environment) -> Result<Value> {
        // For the minimal MVP to make the tests pass:
        // We will do simple matching on standard AST nodes
        use crate::parser::ast::Expression;

        match expr {
            Expression::BooleanLiteral(b) => Ok(Value::Boolean(b.value)),
            Expression::IntegerLiteral(i) => Ok(Value::Integer(i.value)),
            Expression::StringLiteral(s) => Ok(Value::Text(std::sync::Arc::from(s.value.clone()))),
            Expression::Identifier(id) => {
                if let Some(val) = env.get(&id.value) {
                    Ok(val.clone())
                } else {
                    Err(Error::internal(format!("Variable not found: {}", id.value)))
                }
            }
            Expression::Infix(comp) => {
                let left = self.eval_expr(&comp.left, env)?;
                let right = self.eval_expr(&comp.right, env)?;
                match comp.operator.as_str() {
                    "<" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Boolean(l < r))
                        } else {
                            Ok(Value::Boolean(false))
                        }
                    }
                    "<=" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Boolean(l <= r))
                        } else {
                            Ok(Value::Boolean(false))
                        }
                    }
                    ">" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Boolean(l > r))
                        } else {
                            Ok(Value::Boolean(false))
                        }
                    }
                    ">=" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Boolean(l >= r))
                        } else {
                            Ok(Value::Boolean(false))
                        }
                    }
                    "=" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Boolean(l == r))
                        } else {
                            Ok(Value::Boolean(false))
                        }
                    }
                    "!=" | "<>" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Boolean(l != r))
                        } else {
                            Ok(Value::Boolean(false))
                        }
                    }
                    "+" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Integer(l + r))
                        } else {
                            Err(Error::internal(
                                "Addition supported only for integers in MVP",
                            ))
                        }
                    }
                    "-" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Integer(l - r))
                        } else {
                            Err(Error::internal(
                                "Subtraction supported only for integers in MVP",
                            ))
                        }
                    }
                    "*" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Integer(l * r))
                        } else {
                            Err(Error::internal(
                                "Multiplication supported only for integers in MVP",
                            ))
                        }
                    }
                    _ => Err(Error::internal(
                        "Operator not implemented in PL/SQL interpreter yet",
                    )),
                }
            }
            _ => Err(Error::internal(format!(
                "Expression type not fully supported in simple PL/SQL interpreter: {:?}",
                expr
            ))),
        }
    }

    fn substitute_variables_in_statement(
        &self,
        stmt: &mut crate::parser::ast::Statement,
        env: &Environment,
    ) {
        match stmt {
            crate::parser::ast::Statement::Insert(insert) => {
                if let Some(select) = &mut insert.select {
                    self.substitute_variables_in_select(select, env);
                } else {
                    for row in &mut insert.values {
                        for expr in row {
                            self.substitute_variables_in_expr(expr, env);
                        }
                    }
                }
            }
            crate::parser::ast::Statement::Update(update) => {
                for expr in update.updates.values_mut() {
                    self.substitute_variables_in_expr(expr, env);
                }
                if let Some(where_expr) = &mut update.where_clause {
                    self.substitute_variables_in_expr(where_expr, env);
                }
            }
            crate::parser::ast::Statement::Delete(delete) => {
                if let Some(where_expr) = &mut delete.where_clause {
                    self.substitute_variables_in_expr(where_expr, env);
                }
            }
            crate::parser::ast::Statement::Select(select) => {
                self.substitute_variables_in_select(select, env);
            }
            _ => {} // Other statements (DDL) typically don't have expressions with variables
        }
    }

    fn substitute_variables_in_select(
        &self,
        select: &mut crate::parser::ast::SelectStatement,
        env: &Environment,
    ) {
        // Substitute in SELECT expressions
        for col in &mut select.columns {
            self.substitute_variables_in_expr(col, env);
        }

        // Substitute in WHERE clause
        if let Some(where_clause) = &mut select.where_clause {
            self.substitute_variables_in_expr(where_clause, env);
        }

        // Substitute in HAVING clause
        if let Some(having_clause) = &mut select.having {
            self.substitute_variables_in_expr(having_clause, env);
        }

        // LIMIT and OFFSET
        if let Some(limit) = &mut select.limit {
            self.substitute_variables_in_expr(limit, env);
        }
        if let Some(offset) = &mut select.offset {
            self.substitute_variables_in_expr(offset, env);
        }
    }

    fn substitute_variables_in_expr(
        &self,
        expr: &mut crate::parser::ast::Expression,
        env: &Environment,
    ) {
        use crate::parser::ast::Expression;

        // Recursively substitute
        match expr {
            Expression::Infix(infix) => {
                self.substitute_variables_in_expr(&mut infix.left, env);
                self.substitute_variables_in_expr(&mut infix.right, env);
            }
            Expression::Prefix(prefix) => {
                self.substitute_variables_in_expr(&mut prefix.right, env);
            }
            Expression::FunctionCall(fc) => {
                for arg in &mut fc.arguments {
                    self.substitute_variables_in_expr(arg, env);
                }
            }
            Expression::Identifier(id) => {
                if let Some(val) = env.get(&id.value) {
                    // Found a match! We need to replace the identifier with a literal
                    match val {
                        Value::Integer(i) => {
                            *expr =
                                Expression::IntegerLiteral(crate::parser::ast::IntegerLiteral {
                                    token: crate::parser::token::Token::new(
                                        crate::parser::token::TokenType::Integer,
                                        i.to_string(),
                                        crate::parser::token::Position::default(),
                                    ),
                                    value: *i,
                                });
                        }
                        Value::Text(s) => {
                            *expr = Expression::StringLiteral(crate::parser::ast::StringLiteral {
                                token: crate::parser::token::Token::new(
                                    crate::parser::token::TokenType::String,
                                    format!("'{}'", s),
                                    crate::parser::token::Position::default(),
                                ),
                                value: s.to_string(),
                                type_hint: None,
                            });
                        }
                        Value::Boolean(b) => {
                            *expr =
                                Expression::BooleanLiteral(crate::parser::ast::BooleanLiteral {
                                    token: crate::parser::token::Token::new(
                                        crate::parser::token::TokenType::Keyword,
                                        if *b {
                                            "TRUE".to_string()
                                        } else {
                                            "FALSE".to_string()
                                        },
                                        crate::parser::token::Position::default(),
                                    ),
                                    value: *b,
                                });
                        }
                        _ => {}
                    }
                }
            }
            // Add other nested expressions as needed
            _ => {}
        }
    }

    fn execute_statement(&self, stmt: &PlSqlStatement, env: &mut Environment) -> Result<bool> {
        match stmt {
            PlSqlStatement::Declare(decl) => {
                for v in &decl.declarations {
                    let mut initial_val = Value::Null(crate::core::DataType::Null);
                    if let Some(expr) = &v.default_value {
                        initial_val = self.eval_expr(expr, env)?;
                        println!("Declaring {} = {:?}", v.name, initial_val);
                    } else {
                        // Very basic default initialization based on type name
                        let ty = v.data_type.to_uppercase();
                        if ty.contains("INT") {
                            initial_val = Value::Integer(0);
                        } else if ty.contains("BOOL") {
                            initial_val = Value::Boolean(false);
                        } else if ty.contains("TEXT")
                            || ty.contains("VARCHAR")
                            || ty.contains("CHAR")
                        {
                            initial_val = Value::Text(std::sync::Arc::from(String::new()));
                        }
                    }
                    if env.assign(&v.name, initial_val.clone()).is_err() {
                        env.define_global(&v.name, initial_val);
                    }
                }
                Ok(false)
            }
            PlSqlStatement::Block(block) => {
                // If it is an explicit inner block, push frame. If root block of procedure, we should probably not,
                // but since assign updates outer frames, it is fine.
                env.push_frame("block");
                let res = self.execute(block, env);
                env.pop_frame();
                res?;
                Ok(false)
            }
            PlSqlStatement::Assignment(assign) => {
                let val = self.eval_expr(&assign.expression, env)?;
                println!("Evaluating assign: {} = {:?}", assign.variable, val);
                // Variables bound from CALL are actually defined globally in the env by backend.
                // Assignment updates them correctly. If not found, fall back to current frame.
                if env.assign(&assign.variable, val.clone()).is_err() {
                    env.define(&assign.variable, val);
                }
                Ok(false)
            }
            PlSqlStatement::If(if_stmt) => {
                let condition_val = self.eval_expr(&if_stmt.condition, env)?;

                let is_true = match condition_val {
                    Value::Boolean(b) => b,
                    _ => false, // Simplification
                };

                if is_true {
                    env.push_frame("if_then");
                    for stmt in &if_stmt.then_block {
                        if self.execute_statement(stmt, env)? {
                            env.pop_frame();
                            return Ok(true);
                        }
                    }
                    env.pop_frame();
                } else if let Some(else_block) = &if_stmt.else_block {
                    env.push_frame("if_else");
                    for stmt in else_block {
                        if self.execute_statement(stmt, env)? {
                            env.pop_frame();
                            return Ok(true);
                        }
                    }
                    env.pop_frame();
                }

                Ok(false)
            }
            PlSqlStatement::While(while_stmt) => {
                env.push_frame("while");
                loop {
                    let condition_val = self.eval_expr(&while_stmt.condition, env)?;
                    let is_true = match condition_val {
                        Value::Boolean(b) => b,
                        _ => false,
                    };

                    if !is_true {
                        break;
                    }

                    for stmt in &while_stmt.block {
                        if self.execute_statement(stmt, env)? {
                            env.pop_frame();
                            return Ok(true);
                        }
                    }
                }
                env.pop_frame();
                Ok(false)
            }
            PlSqlStatement::Sql(box_stmt) => {
                println!("Executing SQL statement in plsql: {:?}", box_stmt);
                if let Some(runner) = self.runner {
                    let mut modified_stmt = *box_stmt.clone();
                    self.substitute_variables_in_statement(&mut modified_stmt, env);

                    // Note: for queries that modify data, we should also track ROW_COUNT,
                    // but for now we'll just execute it.
                    runner.execute_ast(&modified_stmt)?;
                    Ok(false)
                } else {
                    Err(Error::internal(
                        "Cannot execute SQL statement: No SqlRunner bridge provided",
                    ))
                }
            }
            PlSqlStatement::Return(_) => Ok(true),
        }
    }
}
