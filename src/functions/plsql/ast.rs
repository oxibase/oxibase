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

/// A PL/SQL statement
#[derive(Debug, Clone, PartialEq)]
pub enum PlSqlStatement {
    /// BEGIN ... END block
    Block(BlockStatement),
    /// Variable assignment: var := expr;
    Assignment(AssignmentStatement),
    /// IF ... THEN ... ELSE ... END IF;
    If(IfStatement),
    /// Standard SQL Statement (INSERT, UPDATE, DELETE, etc)
    Sql(Box<crate::parser::ast::Statement>),
    /// RETURN statement
    Return,
}

/// A block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatement {
    pub statements: Vec<PlSqlStatement>,
}

/// Variable assignment
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentStatement {
    pub variable: String,
    pub expression: Expression,
}

/// IF statement
#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Vec<PlSqlStatement>,
    pub else_block: Option<Vec<PlSqlStatement>>,
}
