#!/bin/bash
sed -i '' '/fn parse_values_table_source/i \
    /// Parse a function table source (table-valued function in FROM clause)\
    /// e.g., generate_series(1, 10) AS gs(value)\
    fn parse_function_table_source(\
        \&mut self,\
        token: Token,\
        name: Identifier,\
    ) -> Option<Expression> {\
        self.next_token(); // consume '\''('\''\
        self.next_token(); // advance to first arg or '\'')'\''\
\
        let mut arguments = Vec::new();\
\
        if !self.cur_token_is(TokenType::Punctuator) || self.cur_token.literal != ")" {\
            arguments = self.parse_expression_list();\
        }\
\
        if !self.cur_token_is(TokenType::Punctuator) || self.cur_token.literal != ")" {\
            self.add_error(format!(\
                "expected '\\')\\'' after function arguments, got {:?} at {}",\
                self.cur_token.token_type, self.cur_token.position\
            ));\
            return None;\
        }\
\
        let mut alias = None;\
        let mut column_aliases = Vec::new();\
\
        if self.peek_token_is_keyword("AS") {\
            self.next_token(); // consume AS\
            if !self.expect_peek(TokenType::Identifier) {\
                return None;\
            }\
            alias = Some(Identifier::new(\
                self.cur_token.clone(),\
                self.cur_token.literal.clone(),\
            ));\
        } else if self.peek_token_is(TokenType::Identifier) {\
            self.next_token(); // consume identifier\
            alias = Some(Identifier::new(\
                self.cur_token.clone(),\
                self.cur_token.literal.clone(),\
            ));\
        }\
\
        if alias.is_some() && self.peek_token_is_punctuator("(") {\
            self.next_token(); // consume '\''('\''\
            self.next_token(); // move to first column alias\
\
            while !self.cur_token_is(TokenType::Punctuator) || self.cur_token.literal != ")" {\
                if !self.cur_token_is(TokenType::Identifier) {\
                    self.add_error(format!(\
                        "expected identifier for column alias, got {:?} at {}",\
                        self.cur_token.token_type, self.cur_token.position\
                    ));\
                    return None;\
                }\
                column_aliases.push(Identifier::new(\
                    self.cur_token.clone(),\
                    self.cur_token.literal.clone(),\
                ));\
\
                self.next_token();\
                if self.cur_token_is(TokenType::Punctuator) && self.cur_token.literal == "," {\
                    self.next_token();\
                }\
            }\
        }\
\
        Some(Expression::FunctionTableSource(\
            FunctionTableSource {\
                token,\
                function: name,\
                arguments,\
                alias,\
                column_aliases,\
            },\
        ))\
    }\
' src/parser/statements.rs
