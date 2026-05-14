# Research: Multiline Code Blocks

## 1. Syntax to Support
**Decision**: Support both Postgres-style dollar quotes (`$$ code $$`, `$tag$ code $tag$`) and Markdown-style triple backticks (`` ``` code ``` ``).
**Rationale**: 
- Dollar quoting is the standard in Postgres and widely understood by database users for escaping code in `CREATE FUNCTION`/`CREATE PROCEDURE`. 
- Triple backticks are widely used in markdown and modern developer tools, providing an excellent and familiar UX for developers copying/pasting snippets.
**Alternatives considered**: Only supporting dollar quotes (standard but dated) or only triple backticks (non-standard for SQL). Supporting both provides the best of both worlds.

## 2. Token Representation in Lexer/Parser
**Decision**: Introduce a new `TokenType::RawString` (or `MultilineString`) instead of reusing `TokenType::String`.
**Rationale**: 
- Currently, `TokenType::String` literals keep their surrounding single quotes (`'`), and `parse_string_literal()` strips them and processes escape sequences (like `\n`, `\'`).
- By introducing `TokenType::RawString`, the lexer can strip the dollar/backtick tags completely and hand over the exact raw inner text. The parser can then just take `Token.literal` verbatim without attempting to strip boundaries or process escape sequences. This strictly satisfies FR-002 ("treat the enclosed content as a raw string literal").
**Alternatives considered**: Reusing `TokenType::String` and keeping the dollar tags, then making `parse_string_literal` detect if the string starts with `$` or `\``. This would mix concerns and complicate the parser's string literal handling.

## 3. Detecting Triple Backticks
**Decision**: In the lexer's `next_token` match for `'`'`, check `self.input` directly to peek two characters ahead (since `peek_char()` only looks one character ahead). If it matches ``` `` ```, parse as a triple-backtick raw string. Otherwise, fall back to `read_quoted_identifier('`')`.
**Rationale**: MySQL-style quoted identifiers use a single backtick. We must unambiguously differentiate between identifier quotes and triple backticks.

## 4. Detecting Dollar Quotes
**Decision**: Update the match for `'$'` in `next_token`. If the next character is a digit, it's a `TokenType::Parameter` (e.g., `$1`). Otherwise, parse it as a dollar-quoted string by reading the optional tag until the second `$` is found, then consuming all characters until the matching closing tag is encountered.
**Rationale**: Safely integrates with existing `$1` parameter support without breaking it.

## 5. Error Handling
**Decision**: If EOF is reached while reading a `RawString`, set `self.last_error` to "unterminated block string literal" (or similar) and return an `Error` token.
**Rationale**: Satisfies FR-005. The lexer already uses `self.last_error` for unterminated single-quoted strings.
