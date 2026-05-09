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
    pub fn new(function_registry: Arc<FunctionRegistry>, runner: Option<&'a dyn crate::functions::backends::SqlRunner>) -> Self {
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
                println!("Comparing {:?} {} {:?}", left, comp.operator, right);
                match comp.operator.as_str() {
                    ">" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Boolean(l > r))
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

    fn substitute_variables_in_statement(&self, stmt: &mut crate::parser::ast::Statement, env: &Environment) {
        // Very basic MVP substitution for INSERT statements to satisfy US4
        
        if let crate::parser::ast::Statement::Insert(insert) = stmt {
            if let Some(_select) = &mut insert.select {
                // If the insert uses a select, we'd need to walk it too
            } else {
                // We're dealing with VALUES
                for row in &mut insert.values {
                    for expr in row {
                        self.substitute_variables_in_expr(expr, env);
                    }
                }
            }
            
        } else if let crate::parser::ast::Statement::Update(update) = stmt {
            for (_, expr) in &mut update.updates {
                self.substitute_variables_in_expr(expr, env);
            }
            if let Some(where_expr) = &mut update.where_clause {
                self.substitute_variables_in_expr(where_expr, env);
            }
        }
        // Delete, Select, etc would follow similarly
    }

    fn substitute_variables_in_expr(&self, expr: &mut crate::parser::ast::Expression, env: &Environment) {
        use crate::parser::ast::Expression;
        
        let mut replace_with = None;
        if let Expression::Identifier(id) = expr {
            if let Some(val) = env.get(&id.value) {
                // Found a match! We need to replace the identifier with a literal
                match val {
                    Value::Integer(i) => {
                        replace_with = Some(Expression::IntegerLiteral(crate::parser::ast::IntegerLiteral {
                            token: crate::parser::token::Token::new(crate::parser::token::TokenType::Integer, i.to_string(), crate::parser::token::Position::default()),
                            value: *i,
                        }));
                    }
                    Value::Text(s) => {
                        replace_with = Some(Expression::StringLiteral(crate::parser::ast::StringLiteral {
                            token: crate::parser::token::Token::new(crate::parser::token::TokenType::String, format!("'{}'", s), crate::parser::token::Position::default()),
                            value: s.to_string(),
                            type_hint: None,
                        }));
                    }
                    Value::Boolean(b) => {
                        replace_with = Some(Expression::BooleanLiteral(crate::parser::ast::BooleanLiteral {
                            token: crate::parser::token::Token::new(crate::parser::token::TokenType::Keyword, if *b { "TRUE".to_string() } else { "FALSE".to_string() }, crate::parser::token::Position::default()),
                            value: *b,
                        }));
                    }
                    _ => {}
                }
            }
        }
        
        if let Some(new_expr) = replace_with {
            *expr = new_expr;
            return;
        }

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
            // Add other nested expressions as needed
            _ => {}
        }
    }

    /// Returns Ok(true) if a RETURN statement was hit, Ok(false) otherwise
    fn execute_statement(&self, stmt: &PlSqlStatement, env: &mut Environment) -> Result<bool> {
        match stmt {
            PlSqlStatement::Block(block) => {
                self.execute(block, env)?;
                Ok(false)
            }
            PlSqlStatement::Assignment(assign) => {
                let val = self.eval_expr(&assign.expression, env)?;
                println!("Assigning {} = {:?}", assign.variable, val);
                // In PL/SQL, variables are case-insensitive. We should just insert/update them.
                env.define(&assign.variable, val);
                println!(
                    "Variable {} now has value {:?}",
                    assign.variable,
                    env.get(&assign.variable)
                );
                Ok(false)
            }
            PlSqlStatement::If(if_stmt) => {
                let condition_val = self.eval_expr(&if_stmt.condition, env)?;
                println!("Condition evaluated to: {:?}", condition_val);

                let is_true = match condition_val {
                    Value::Boolean(b) => b,
                    _ => false, // Simplification
                };

                if is_true {
                    for stmt in &if_stmt.then_block {
                        if self.execute_statement(stmt, env)? {
                            return Ok(true);
                        }
                    }
                } else if let Some(else_block) = &if_stmt.else_block {
                    for stmt in else_block {
                        if self.execute_statement(stmt, env)? {
                            return Ok(true);
                        }
                    }
                }

                Ok(false)
            }
            PlSqlStatement::Sql(sql_stmt) => {
                println!("Executing SQL statement: {}", sql_stmt);
                if let Some(runner) = self.runner {
                    // Inject variables before execution
                    // A real implementation would parse the AST, substitute identifiers matching variables
                    // For this MVP we just execute it directly (which works if the query doesn't depend on vars)
                    
                    // Implement variable substitution (injecting PL/SQL variables into the standard SQL AST before execution)
                    let mut modified_stmt = sql_stmt.clone();
                    self.substitute_variables_in_statement(&mut modified_stmt, env);
                    
                    runner.execute_ast(&modified_stmt)?;
                    Ok(false)
                } else {
                    Err(Error::internal("Cannot execute SQL statement: No SqlRunner bridge provided"))
                }
            }
            PlSqlStatement::Return => Ok(true),
        }
    }
}
