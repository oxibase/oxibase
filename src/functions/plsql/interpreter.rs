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

use crate::core::{Error, Result, Row, Schema, Value};
use crate::executor::context::ExecutionContext;
use crate::executor::expression::ExpressionEval;
use super::ast::{BlockStatement, PlSqlStatement};
use super::env::Environment;
use std::sync::Arc;
use crate::functions::FunctionRegistry;

pub struct PlSqlInterpreter {
    function_registry: Arc<FunctionRegistry>,
}

impl PlSqlInterpreter {
    pub fn new(function_registry: Arc<FunctionRegistry>) -> Self {
        Self { function_registry }
    }

    pub fn execute(
        &self,
        block: &BlockStatement,
        env: &mut Environment,
    ) -> Result<()> {
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
            },
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
                    },
                    "=" => {
                        if let (Value::Integer(l), Value::Integer(r)) = (&left, &right) {
                            Ok(Value::Boolean(l == r))
                        } else {
                            Ok(Value::Boolean(false))
                        }
                    },
                    _ => Err(Error::internal("Operator not implemented in PL/SQL interpreter yet"))
                }
            },
            _ => Err(Error::internal(format!("Expression type not fully supported in simple PL/SQL interpreter: {:?}", expr)))
        }
    }

    /// Returns Ok(true) if a RETURN statement was hit, Ok(false) otherwise
    fn execute_statement(
        &self,
        stmt: &PlSqlStatement,
        env: &mut Environment,
    ) -> Result<bool> {
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
                println!("Variable {} now has value {:?}", assign.variable, env.get(&assign.variable));
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
            PlSqlStatement::Return => Ok(true),
        }
    }
}
