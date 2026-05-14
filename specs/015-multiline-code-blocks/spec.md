# Feature Specification: Multiline Code Blocks

**Feature Branch**: `015-multiline-code-blocks`  
**Created**: 2026-05-14  
**Status**: Draft  
**Input**: User description: "in the functions and procedures, i would like to be able to use $ or ``` or any other block start and block end way of signaling that inside there is code in another language. Can you get inspired by postgres and others ?"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create Function with Postgres-style Dollar Quoting (Priority: P1)

As a database user writing stored procedures or functions, I want to use dollar-quoted strings (e.g., `$$ code $$` or `$python$ code $python$`) so that I don't have to escape single quotes within my foreign language code.

**Why this priority**: Stored procedures and script engines are already supported, but embedding code with single quotes is tedious and error-prone. This improves developer experience significantly.

**Independent Test**: Can be fully tested by a new integration test parsing a `CREATE FUNCTION` statement with dollar-quoted strings.

**Acceptance Scenarios**:

1. **Given** an empty database, **When** the user executes a `CREATE FUNCTION` command using `$$` to enclose the function body, **Then** the function is created successfully and the exact body is preserved.
2. **Given** an empty database, **When** the user executes a `CREATE FUNCTION` command using tagged dollar quotes like `$py$` and `$py$`, **Then** the body is correctly parsed until the matching tag.

---

### User Story 2 - Alternative Block Delimiters (Triple Backticks) (Priority: P2)

As a database user familiar with Markdown, I want to optionally use triple backticks (` ``` `) to enclose code blocks, so that I can easily copy-paste code snippets from documentation into my SQL statements.

**Why this priority**: While dollar quoting is standard for databases, supporting backticks provides a modern, developer-friendly alternative.

**Independent Test**: Can be tested by adding parser tests that recognize backtick-enclosed strings as valid string literals in place of single-quoted strings.

**Acceptance Scenarios**:

1. **Given** a SQL interface, **When** the user writes a string enclosed in triple backticks, **Then** the parser recognizes it as a valid string literal block.

### Edge Cases

- What happens when the end tag does not match the start tag (e.g., `$a$ body $b$`)?
- How does the system handle nested dollar quotes (e.g., `$a$ body containing $$ inner $$ $a$`)?
- What happens when a file ends before the closing block delimiter is found?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The SQL parsing engine MUST support both Postgres-style dollar quotes (`$tag$ ... $tag$`) and Markdown-style triple backticks (` ``` ... ``` `) as block quote syntaxes.
- **FR-002**: The engine MUST treat the enclosed content as a raw string literal without requiring escaping of internal single quotes.
- **FR-003**: For dollar quotes, the engine MUST support an optional tag between the dollars (e.g., `$tag$`) and match it with the identical closing tag.
- **FR-004**: The system MUST allow these multiline string literals wherever a standard string literal is currently accepted (especially in `CREATE FUNCTION` and `CREATE PROCEDURE` statements).
- **FR-005**: If the end of input is reached before the closing delimiter, the system MUST return a syntax error indicating an unterminated string block.

### Key Entities 

- **Multiline String Literal**: A new abstract representation of the code block text in the query structure.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully define functions in supported scripting languages using dollar-quoted bodies containing unescaped single quotes.
- **SC-002**: All existing query parsing tests pass without regression (the new syntax does not break standard string parsing).
- **SC-003**: The system correctly identifies and reports syntax errors for unclosed block statements.

## Assumptions

- The primary use case is `CREATE FUNCTION` and `CREATE PROCEDURE`, but the syntax will be implemented at the lexer level as a general string literal alternative.
- Foreign language block detection will simply treat the enclosed body as a standard `String` value in the AST.
