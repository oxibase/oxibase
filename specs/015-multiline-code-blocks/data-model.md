# Data Model: Multiline Code Blocks

## Token Additions

```rust
// In src/parser/token.rs

pub enum TokenType {
    // ... existing types
    
    /// Raw string literal (e.g. dollar-quoted or triple-backticks)
    /// The literal value will NOT contain the surrounding tags, 
    /// and should be used exactly as-is without unescaping.
    RawString,
}
```

## Expression Additions

No new `Expression` enum variants are strictly needed because `Expression::StringLiteral` already represents string values. 

However, `parse_prefix` in the Pratt parser will need to map `TokenType::RawString` to a string literal parser function:

```rust
// In src/parser/expressions.rs

    fn parse_prefix(&mut self) -> Option<Expression> {
        match self.cur_token.token_type {
            // ... existing
            TokenType::String => Some(self.parse_string_literal()),
            TokenType::RawString => Some(self.parse_raw_string_literal()),
            // ... existing
        }
    }

    /// Parses a raw string literal directly from the token literal without 
    /// stripping boundary quotes or applying escape sequences.
    fn parse_raw_string_literal(&self) -> Expression {
        let value = self.cur_token.literal.clone();
        
        let type_hint = if value.starts_with('{') && value.ends_with('}') {
            Some(crate::core::Type::Json)
        } else {
            None
        };
        
        Expression::StringLiteral(StringLiteral {
            token: self.cur_token.clone(),
            value,
            type_hint,
        })
    }
```
