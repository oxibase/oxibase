use strict;
use warnings;

my $methods = <<'METHODS';

    // ========================================================================
    // Sequences
    // ========================================================================

    fn parse_create_sequence_statement(&mut self) -> Option<CreateSequenceStatement> {
        let token = self.cur_token.clone();

        let if_not_exists = if self.peek_token_is_keyword("IF") {
            self.next_token(); // CONSUME IF
            if self.expect_keyword("NOT") && self.expect_keyword("EXISTS") {
                true
            } else {
                return None;
            }
        } else {
            false
        };

        let name = self.parse_table_name()?;

        let mut start_with = None;
        let mut increment_by = None;
        let mut min_value = None;
        let mut max_value = None;
        let mut cycle = false;

        while !self.cur_token_is(TokenType::Eof) && !self.cur_token_is(TokenType::Semicolon) {
            if self.peek_token_is_keyword("START") {
                self.next_token(); // CONSUME START
                if self.peek_token_is_keyword("WITH") {
                    self.next_token(); // CONSUME WITH
                }
                if !self.expect_peek(TokenType::Integer) {
                    return None;
                }
                start_with = Some(self.cur_token.literal.parse::<i64>().unwrap_or(1));
            } else if self.peek_token_is_keyword("INCREMENT") {
                self.next_token(); // CONSUME INCREMENT
                if self.peek_token_is_keyword("BY") {
                    self.next_token(); // CONSUME BY
                }
                
                // Allow negative numbers
                let is_negative = if self.peek_token_is(TokenType::Minus) {
                    self.next_token();
                    true
                } else {
                    false
                };
                
                if !self.expect_peek(TokenType::Integer) {
                    return None;
                }
                let val = self.cur_token.literal.parse::<i64>().unwrap_or(1);
                increment_by = Some(if is_negative { -val } else { val });
            } else if self.peek_token_is_keyword("MINVALUE") {
                self.next_token(); // CONSUME MINVALUE
                
                let is_negative = if self.peek_token_is(TokenType::Minus) {
                    self.next_token();
                    true
                } else {
                    false
                };
                
                if !self.expect_peek(TokenType::Integer) {
                    return None;
                }
                let val = self.cur_token.literal.parse::<i64>().unwrap_or(1);
                min_value = Some(if is_negative { -val } else { val });
            } else if self.peek_token_is_keyword("NO") {
                self.next_token(); // CONSUME NO
                if self.peek_token_is_keyword("MINVALUE") {
                    self.next_token();
                    min_value = None;
                } else if self.peek_token_is_keyword("MAXVALUE") {
                    self.next_token();
                    max_value = None;
                } else if self.peek_token_is_keyword("CYCLE") {
                    self.next_token();
                    cycle = false;
                } else {
                    self.add_error(format!("Unexpected token after NO: {}", self.peek_token.literal));
                    return None;
                }
            } else if self.peek_token_is_keyword("MAXVALUE") {
                self.next_token(); // CONSUME MAXVALUE
                if !self.expect_peek(TokenType::Integer) {
                    return None;
                }
                max_value = Some(self.cur_token.literal.parse::<i64>().unwrap_or(i64::MAX));
            } else if self.peek_token_is_keyword("CYCLE") {
                self.next_token(); // CONSUME CYCLE
                cycle = true;
            } else {
                break;
            }
        }

        Some(CreateSequenceStatement {
            token,
            name,
            if_not_exists,
            start_with,
            increment_by,
            min_value,
            max_value,
            cycle,
        })
    }

    fn parse_alter_sequence_statement(&mut self) -> Option<AlterSequenceStatement> {
        let token = self.cur_token.clone();

        let if_exists = if self.peek_token_is_keyword("IF") {
            self.next_token(); // CONSUME IF
            if self.expect_keyword("EXISTS") {
                true
            } else {
                return None;
            }
        } else {
            false
        };

        let name = self.parse_table_name()?;

        let mut restart_with = None;
        let mut increment_by = None;
        let mut min_value = None;
        let mut max_value = None;
        let mut cycle = None;

        while !self.cur_token_is(TokenType::Eof) && !self.cur_token_is(TokenType::Semicolon) {
            if self.peek_token_is_keyword("RESTART") {
                self.next_token(); // CONSUME RESTART
                if self.peek_token_is_keyword("WITH") {
                    self.next_token(); // CONSUME WITH
                }
                if !self.expect_peek(TokenType::Integer) {
                    return None;
                }
                restart_with = Some(self.cur_token.literal.parse::<i64>().unwrap_or(1));
            } else if self.peek_token_is_keyword("INCREMENT") {
                self.next_token(); // CONSUME INCREMENT
                if self.peek_token_is_keyword("BY") {
                    self.next_token(); // CONSUME BY
                }
                
                let is_negative = if self.peek_token_is(TokenType::Minus) {
                    self.next_token();
                    true
                } else {
                    false
                };
                
                if !self.expect_peek(TokenType::Integer) {
                    return None;
                }
                let val = self.cur_token.literal.parse::<i64>().unwrap_or(1);
                increment_by = Some(if is_negative { -val } else { val });
            } else if self.peek_token_is_keyword("MINVALUE") {
                self.next_token(); // CONSUME MINVALUE
                let is_negative = if self.peek_token_is(TokenType::Minus) {
                    self.next_token();
                    true
                } else {
                    false
                };
                if !self.expect_peek(TokenType::Integer) {
                    return None;
                }
                let val = self.cur_token.literal.parse::<i64>().unwrap_or(1);
                min_value = Some(if is_negative { -val } else { val });
            } else if self.peek_token_is_keyword("NO") {
                self.next_token(); // CONSUME NO
                if self.peek_token_is_keyword("MINVALUE") {
                    self.next_token();
                    // Just set to None to represent NO MINVALUE, though Alter semantics may require explicit struct handling. We will use None for NO MINVALUE.
                } else if self.peek_token_is_keyword("MAXVALUE") {
                    self.next_token();
                } else if self.peek_token_is_keyword("CYCLE") {
                    self.next_token();
                    cycle = Some(false);
                } else {
                    self.add_error(format!("Unexpected token after NO: {}", self.peek_token.literal));
                    return None;
                }
            } else if self.peek_token_is_keyword("MAXVALUE") {
                self.next_token(); // CONSUME MAXVALUE
                if !self.expect_peek(TokenType::Integer) {
                    return None;
                }
                max_value = Some(self.cur_token.literal.parse::<i64>().unwrap_or(i64::MAX));
            } else if self.peek_token_is_keyword("CYCLE") {
                self.next_token(); // CONSUME CYCLE
                cycle = Some(true);
            } else {
                break;
            }
        }

        Some(AlterSequenceStatement {
            token,
            name,
            if_exists,
            restart_with,
            increment_by,
            min_value,
            max_value,
            cycle,
        })
    }

    fn parse_drop_sequence_statement(&mut self) -> Option<DropSequenceStatement> {
        let token = self.cur_token.clone();

        let if_exists = if self.peek_token_is_keyword("IF") {
            self.next_token(); // CONSUME IF
            if self.expect_keyword("EXISTS") {
                true
            } else {
                return None;
            }
        } else {
            false
        };

        let name = self.parse_table_name()?;

        Some(DropSequenceStatement {
            token,
            name,
            if_exists,
        })
    }
METHODS

open my $fh, '<', 'src/parser/statements.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/}\n$/$methods}\n/;

open $fh, '>', 'src/parser/statements.rs' or die $!;
print $fh $content;
close $fh;
