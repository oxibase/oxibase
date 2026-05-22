#!/bin/bash
sed -i '' '/if !self.cur_token_is(TokenType::Identifier) && !self.cur_token_is(TokenType::Keyword) {/i \
        // Check if it is a function call like generate_series(1, 10)\
        if self.cur_token_is(TokenType::Identifier) && self.peek_token_is_punctuator("(") {\
            let token = self.cur_token.clone();\
            let function_name = Identifier::new(token.clone(), self.cur_token.literal.clone());\
            return self.parse_function_table_source(token, function_name);\
        }\
' src/parser/statements.rs
