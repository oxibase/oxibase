# Implementation Tasks: Multiline Code Blocks

**Feature**: Multiline Code Blocks
**Spec**: [spec.md](./spec.md)
**Plan**: [plan.md](./plan.md)

## Dependencies

- **US1 (Postgres-style Dollar Quoting)** depends on Foundational Phase
- **US2 (Triple Backticks)** depends on Foundational Phase
- US1 and US2 can be implemented in parallel.

## Phase 1: Setup

*(No setup required for this feature)*

## Phase 2: Foundational

**Goal**: Extend the Token representation to support Raw Strings.

- [x] T001 Extend `TokenType` enum in `src/parser/token.rs` to add a new `RawString` variant

## Phase 3: Postgres-style Dollar Quoting (US1)

**Goal**: Support dollar quotes (`$$` or `$tag$`) in the Lexer and Parser.
**Independent Test**: Lexer produces `RawString` tokens for dollar quotes; Parser accepts them as string literals.

- [x] T002 [US1] Add parser tests in `src/parser/expressions.rs` for dollar-quoted string literals without tags
- [x] T003 [US1] Add parser tests in `src/parser/expressions.rs` for dollar-quoted string literals with tags
- [x] T004 [US1] Implement `read_dollar_quoted_string` method in `Lexer` (`src/parser/lexer.rs`) handling optional tags and matching closing tags
- [x] T005 [US1] Modify Lexer `next_token` in `src/parser/lexer.rs` to detect dollar quotes without breaking parameter (`$1`) parsing
- [x] T006 [US1] Add error handling in `Lexer::read_dollar_quoted_string` for EOF before the closing tag
- [x] T007 [US1] Update `parse_prefix` in `src/parser/expressions.rs` to map `TokenType::RawString` to a new `parse_raw_string_literal` function
- [x] T008 [US1] Implement `parse_raw_string_literal` in `src/parser/expressions.rs`

## Phase 4: Alternative Block Delimiters (US2)

**Goal**: Support markdown-style triple backticks (```...```).
**Independent Test**: Lexer produces `RawString` tokens for triple backticks; parser tests pass.

- [x] T009 [US2] Add parser tests in `src/parser/expressions.rs` for triple backtick string literals
- [x] T010 [US2] Implement `read_triple_backtick_string` method in `Lexer` (`src/parser/lexer.rs`)
- [x] T011 [US2] Modify Lexer `next_token` in `src/parser/lexer.rs` for backtick (`\``) to look ahead for two more backticks, routing to `read_triple_backtick_string` or fallback to `read_quoted_identifier`
- [x] T012 [US2] Add error handling in `Lexer::read_triple_backtick_string` for EOF before closing backticks

## Phase 5: Polish & Cross-Cutting

**Goal**: Final cleanups and end-to-end integration tests.

- [x] T013 [P] Add End-to-End integration test in `tests/` verifying `CREATE FUNCTION` with a multiline python script using dollar quotes
- [x] T014 [P] Verify performance/memory compliance (run `make test` and `make lint` to ensure no warnings or unwrap usage)
