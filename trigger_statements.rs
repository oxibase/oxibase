    /// Parse a CREATE TRIGGER statement
    fn parse_create_trigger_statement(&mut self) -> Option<CreateTriggerStatement> {
        let token = self.cur_token.clone();

        let if_not_exists = if self.peek_token_is_keyword("IF") {
            self.next_token();
            if !self.expect_keyword("NOT") {
                return None;
            }
            if !self.expect_keyword("EXISTS") {
                return None;
            }
            true
        } else {
            false
        };

        if !self.expect_peek(TokenType::Identifier) {
            return None;
        }
        let trigger_name = Identifier::new(self.cur_token.clone(), self.cur_token.literal.clone());

        self.next_token();
        let timing = if self.cur_token_is_keyword("BEFORE") {
            TriggerTiming::Before
        } else if self.cur_token_is_keyword("AFTER") {
            TriggerTiming::After
        } else {
            self.add_error(format!("expected BEFORE or AFTER at {}", self.cur_token.position));
            return None;
        };

        self.next_token();
        let event = if self.cur_token_is_keyword("INSERT") {
            TriggerEvent::Insert
        } else if self.cur_token_is_keyword("UPDATE") {
            TriggerEvent::Update
        } else if self.cur_token_is_keyword("DELETE") {
            TriggerEvent::Delete
        } else {
            self.add_error(format!("expected INSERT, UPDATE, or DELETE at {}", self.cur_token.position));
            return None;
        };

        if !self.expect_keyword("ON") {
            return None;
        }

        let table_name = self.parse_table_name()?;

        let mut for_each_row = false;
        if self.peek_token_is_keyword("FOR") {
            self.next_token();
            if !self.expect_keyword("EACH") {
                return None;
            }
            if !self.expect_keyword("ROW") {
                return None;
            }
            for_each_row = true;
        }

        if !self.expect_keyword("LANGUAGE") {
            return None;
        }

        if !self.expect_peek(TokenType::Identifier) && !self.expect_peek(TokenType::Keyword) {
            self.add_error(format!("expected language name at {}", self.cur_token.position));
            return None;
        }
        let language = self.cur_token.literal.clone();

        if !self.expect_keyword("AS") {
            return None;
        }

        if !self.expect_peek(TokenType::String) {
            self.add_error(format!("expected procedure body string at {}", self.cur_token.position));
            return None;
        }
        let body = self.cur_token.literal.trim_matches('\'').to_string();

        Some(CreateTriggerStatement {
            token,
            trigger_name,
            if_not_exists,
            timing,
            event,
            table_name,
            for_each_row,
            language,
            body,
        })
    }

    /// Parse a DROP TRIGGER statement
    fn parse_drop_trigger_statement(&mut self) -> Option<DropTriggerStatement> {
        let token = self.cur_token.clone();

        let if_exists = if self.peek_token_is_keyword("IF") {
            self.next_token();
            if !self.expect_keyword("EXISTS") {
                return None;
            }
            true
        } else {
            false
        };

        if !self.expect_peek(TokenType::Identifier) {
            return None;
        }
        let trigger_name = Identifier::new(self.cur_token.clone(), self.cur_token.literal.clone());

        if !self.expect_keyword("ON") {
            return None;
        }

        let table_name = self.parse_table_name()?;

        Some(DropTriggerStatement {
            token,
            trigger_name,
            table_name,
            if_exists,
        })
    }
