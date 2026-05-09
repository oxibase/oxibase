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

use crate::parser::ast::Expression;
use crate::parser::token::Token;

/// A PL/SQL statement
#[derive(Debug, Clone, PartialEq)]
pub enum PlSqlStatement {
    /// DECLARE block
    Declare(DeclareStatement),
    /// BEGIN ... END block
    Block(BlockStatement),
    /// Variable assignment: var := expr;
    Assignment(AssignmentStatement),
    /// IF ... THEN ... ELSE ... END IF;
    If(IfStatement),
    /// WHILE ... LOOP ... END LOOP;
    While(WhileStatement),
    /// Standard SQL Statement (INSERT, UPDATE, DELETE, etc)
    Sql(Box<crate::parser::ast::Statement>),
    /// RETURN statement
    Return(Token),
}

/// A variable declaration
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    pub name: String,
    pub data_type: String,
    pub default_value: Option<Expression>,
}

/// A declare statement
#[derive(Debug, Clone, PartialEq)]
pub struct DeclareStatement {
    pub token: Token,
    pub declarations: Vec<VariableDeclaration>,
}

/// A block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<PlSqlStatement>,
}

/// Variable assignment
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentStatement {
    pub token: Token,
    pub variable: String,
    pub expression: Expression,
}

/// IF statement
#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub token: Token,
    pub condition: Expression,
    pub then_block: Vec<PlSqlStatement>,
    pub else_block: Option<Vec<PlSqlStatement>>,
}

/// WHILE statement
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatement {
    pub token: Token,
    pub condition: Expression,
    pub block: Vec<PlSqlStatement>,
}
