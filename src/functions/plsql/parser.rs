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

use super::ast::{AssignmentStatement, BlockStatement, IfStatement, PlSqlStatement};
use crate::core::{Error, Result};
use crate::parser::lexer::Lexer;
use crate::parser::precedence::Precedence;
use crate::parser::token::{Token, TokenType};

pub struct PlSqlParser {
    code: String,
    lexer: Lexer,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

impl PlSqlParser {
    pub fn new(code: &str) -> Self {
        let mut lexer = Lexer::new(code);
        let cur_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Self {
            code: code.to_string(),
            lexer,
            cur_token,
            peek_token,
            errors: Vec::new(),
        }
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    #[allow(dead_code)]
    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token.token_type == t {
            self.next_token();
            true
        } else {
            self.errors.push(format!(
                "Expected next token to be {:?}, got {:?} instead",
                t, self.peek_token.token_type
            ));
            false
        }
    }

    fn peek_is_keyword(&self, keyword: &str) -> bool {
        self.peek_token.token_type == TokenType::Keyword
            && self.peek_token.literal.eq_ignore_ascii_case(keyword)
    }

    fn expect_keyword(&mut self, keyword: &str) -> bool {
        if self.peek_is_keyword(keyword) {
            self.next_token();
            true
        } else {
            self.errors.push(format!(
                "Expected keyword {}, got {}",
                keyword, self.peek_token.literal
            ));
            false
        }
    }

    pub fn parse(&mut self) -> Result<BlockStatement> {
        let mut statements = Vec::new();

        // Skip DECLARE for now, assume BEGIN is the start of executable section
        if self.cur_token.token_type == TokenType::Keyword
            && self.cur_token.literal.eq_ignore_ascii_case("DECLARE")
        {
            // we would parse variable declarations here, but let's skip to BEGIN
            while !(self.cur_token.token_type == TokenType::Keyword
                && self.cur_token.literal.eq_ignore_ascii_case("BEGIN"))
            {
                if self.cur_token.token_type == TokenType::Eof {
                    return Err(Error::parse("Unexpected EOF waiting for BEGIN"));
                }
                self.next_token();
            }
        }

        if self.cur_token.token_type == TokenType::Keyword
            && self.cur_token.literal.eq_ignore_ascii_case("BEGIN")
        {
            self.next_token();

            while !(self.cur_token.token_type == TokenType::Keyword
                && self.cur_token.literal.eq_ignore_ascii_case("END"))
            {
                if self.cur_token.token_type == TokenType::Eof {
                    return Err(Error::parse("Unexpected EOF waiting for END"));
                }

                if let Some(stmt) = self.parse_statement() {
                    statements.push(stmt);
                } else {
                    // We must NOT blindly consume tokens if parse_statement failed
                    // Or actually, if it returned None because it wasn't a statement, we can advance
                    // But if it consumed tokens and failed, we are in trouble.
                    // Wait, parse_assignment_statement consumes tokens and returns Some.
                    // Does it consume the semicolon? Yes.
                    self.next_token();
                }
            }
        } else {
            // Just parse statements directly if no BEGIN
            while self.cur_token.token_type != TokenType::Eof {
                if let Some(stmt) = self.parse_statement() {
                    statements.push(stmt);
                }
                self.next_token();
            }
        }

        if !self.errors.is_empty() {
            return Err(Error::parse(self.errors.join("\n")));
        }

        Ok(BlockStatement { statements })
    }

    fn parse_statement(&mut self) -> Option<PlSqlStatement> {
        match self.cur_token.token_type {
            TokenType::Keyword => {
                let kw = self.cur_token.literal.to_uppercase();
                match kw.as_str() {
                    "IF" => self.parse_if_statement(),
                    "RETURN" => {
                        let stmt = PlSqlStatement::Return;
                        if self.peek_token.literal == ";" {
                            self.next_token();
                        }
                        Some(stmt)
                    }
                    _ => {
                        // Try assignment first
                        if let Some(stmt) = self.parse_assignment_statement() {
                            return Some(stmt);
                        }
                        
                        // Fallback to standard SQL parser
                        self.parse_sql_statement()
                    }
                }
            }
            TokenType::Identifier => {
                // Try assignment first
                if let Some(stmt) = self.parse_assignment_statement() {
                    return Some(stmt);
                }
                
                // Fallback to standard SQL parser
                self.parse_sql_statement()
            },
            _ => self.parse_sql_statement(),
        }
    }

    fn parse_sql_statement(&mut self) -> Option<PlSqlStatement> {
        let mut sql_parser = crate::parser::Parser::new(&self.code[self.cur_token.position.offset..]);
        let program = sql_parser.parse_program();
        
        println!("Trying to parse SQL starting with: {:?}", self.cur_token);
        
        if let Ok(prog) = program {
            if !prog.statements.is_empty() {
                let stmt = prog.statements[0].clone();
                println!("Successfully parsed SQL: {:?}", stmt);
                
                // We need to advance our lexer past the consumed SQL statement
                // A reliable way is to sync our lexer's position with the sql_parser's current token position
                // Wait, the inner parser consumes up to a semicolon.
                // We can just loop until semicolon or EOF.
                while self.cur_token.literal != ";" && self.cur_token.token_type != TokenType::Eof {
                    self.next_token();
                }
                if self.cur_token.literal == ";" {
                    // Semicolons might be optional or consumed by inner parser, let's assume we land on it
                    // Actually let's not consume it here, the caller handles it or the loop will.
                    // Wait, standard SQL statements end with a semicolon. We should consume it.
                    self.next_token();
                }
                
                return Some(PlSqlStatement::Sql(stmt));
            }
        }
        
        None
    }

    fn parse_assignment_statement(&mut self) -> Option<PlSqlStatement> {
        let variable = self.cur_token.literal.clone();

        // Expect := or = or :
        if self.peek_token.literal == "=" {
            self.next_token(); // Move to =
            self.next_token(); // Move to expression start
        } else if self.peek_token.literal == ":" {
            self.next_token(); // Move to :
            if self.peek_token.literal == "=" {
                self.next_token(); // Move to =
                self.next_token(); // Move to expression start
            } else {
                return None;
            }
        } else if self.peek_token.literal == ":=" {
            self.next_token(); // Move to :=
            self.next_token(); // Move to expression start
        } else {
            return None;
        }

        // This is a hacky way to re-use the standard SQL expression parser
        // We'll create a new parser just for the expression part
        let mut sql_parser =
            crate::parser::Parser::new(&self.code[self.cur_token.position.offset..]);

        // Advance our lexer until semicolon
        let mut expr_tokens = Vec::new();
        while self.cur_token.literal != ";" && self.cur_token.token_type != TokenType::Eof {
            expr_tokens.push(self.cur_token.clone());
            self.next_token();
        }

        // If we hit EOF before semicolon, it's an error
        if self.cur_token.token_type == TokenType::Eof && expr_tokens.is_empty() {
            self.errors
                .push("Expected expression after assignment".to_string());
            return None;
        }

        println!("Tokens for assignment to {}: {:?}", variable, expr_tokens);

        // Now parse the expression using the standard parser
        // The problem is sql_parser reads the expression but our main lexer has skipped ahead to the semicolon.
        // Let's make sure sql_parser successfully parsed it.
        if let Some(expr) = sql_parser.parse_expression(Precedence::Lowest) {
            let stmt = PlSqlStatement::Assignment(AssignmentStatement {
                variable,
                expression: expr,
            });
            // Consume semicolon if present
            if self.cur_token.literal == ";" {
                self.next_token();
            }
            // println!("Parsed assignment: {:?}", stmt);
            Some(stmt)
        } else {
            self.errors.push(format!(
                "Failed to parse expression in assignment for {}: {:?}",
                variable,
                sql_parser.errors()
            ));
            None
        }
    }

    fn parse_if_statement(&mut self) -> Option<PlSqlStatement> {
        self.next_token(); // Move past IF

        // Collect tokens until THEN for the condition
        let mut sql_parser =
            crate::parser::Parser::new(&self.code[self.cur_token.position.offset..]);
        let condition = sql_parser.parse_expression(Precedence::Lowest)?;

        // Advance our lexer to THEN
        while !(self.cur_token.token_type == TokenType::Keyword
            && self.cur_token.literal.eq_ignore_ascii_case("THEN"))
        {
            if self.cur_token.token_type == TokenType::Eof {
                self.errors
                    .push("Expected THEN after IF condition".to_string());
                return None;
            }
            self.next_token();
        }

        self.next_token(); // Move past THEN

        let mut then_block = Vec::new();
        let mut else_block = None;

        // Parse THEN block
        while !(self.cur_token.token_type == TokenType::Keyword
            && (self.cur_token.literal.eq_ignore_ascii_case("ELSE")
                || self.cur_token.literal.eq_ignore_ascii_case("END")))
        {
            // Debug parsing statements
            println!("Parsing stmt inside THEN block: {:?}", self.cur_token);
            // println!("IF block token: {:?}", self.cur_token);
            if self.cur_token.token_type == TokenType::Eof {
                self.errors.push("Expected END IF".to_string());
                return None;
            }
            if let Some(stmt) = self.parse_statement() {
                then_block.push(stmt);
            } else {
                self.next_token();
            }
        }

        // Parse optional ELSE block
        if self.cur_token.token_type == TokenType::Keyword
            && self.cur_token.literal.eq_ignore_ascii_case("ELSE")
        {
            self.next_token(); // Move past ELSE
            let mut block = Vec::new();
            while !(self.cur_token.token_type == TokenType::Keyword
                && self.cur_token.literal.eq_ignore_ascii_case("END"))
            {
                if self.cur_token.token_type == TokenType::Eof {
                    self.errors.push("Expected END IF".to_string());
                    return None;
                }
                if let Some(stmt) = self.parse_statement() {
                    block.push(stmt);
                } else {
                    self.next_token();
                }
            }
            else_block = Some(block);
        }

        // Expect END IF
        if self.cur_token.token_type == TokenType::Keyword
            && self.cur_token.literal.eq_ignore_ascii_case("END")
            && self.expect_keyword("IF")
        {
            if self.peek_token.literal == ";" {
                self.next_token(); // Consume semicolon
            }
            return Some(PlSqlStatement::If(IfStatement {
                condition,
                then_block,
                else_block,
            }));
        }

        None
    }
}
