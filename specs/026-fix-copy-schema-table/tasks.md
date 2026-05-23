# Implementation Tasks: Fix COPY Schema-Qualified Table Syntax

**Feature**: Fix COPY Schema-Qualified Table Syntax
**Branch**: `026-fix-copy-schema-table`

## Dependencies

- **US1 (Load Data into Schema-Qualified Table)**: Can be implemented directly. No complex dependencies exist since it relies on modifying existing parser structures.

## Parallel Execution Examples

- **US1**: `T001` (Modify `CopyStatement`), `T002` (Modify `parse_copy_statement`), and `T003` (Update Executor `copy.rs`) must be done largely sequentially as `T001` changes the AST which `T002` and `T003` use, but `T002` and `T003` can be parallelized since one deals with parsing into the AST and the other with reading from it.

## Implementation Strategy

1. **Foundational (T001)**: Modify the `CopyStatement` AST node in `src/parser/ast.rs` to use `TableName` instead of `Identifier`. This breaks compilation.
2. **Parser Update (T002)**: Fix the parser in `src/parser/statements.rs` to use `self.parse_table_name()` and populate the updated `CopyStatement`. Add tests.
3. **Executor Update (T003)**: Update the executor in `src/executor/copy.rs` to use the schema and table name from `TableName` correctly.

---

## Phase 1: Setup

*(No specific setup tasks required for this bug fix.)*

## Phase 2: Foundational

- [x] T001 [P] Modify `CopyStatement` struct in `src/parser/ast.rs` to change the `table_name` field from `Identifier` to `TableName`.
- [x] T002 Update `fmt::Display` implementation for `CopyStatement` in `src/parser/ast.rs` to format the `TableName`.

## Phase 3: User Story 1 - Load Data into Schema-Qualified Table (Priority: P1)

**Story Goal**: Users need to load data from external files directly into schema-qualified tables using `COPY schema.table FROM ...`.

**Independent Test**: `cargo nextest run test_parse_copy_statement` (will be updated) and other existing executor tests for COPY.

- [x] T003 [P] [US1] Update `parse_copy_statement` in `src/parser/statements.rs` to use `self.parse_table_name()` instead of manually parsing a single identifier. Update to construct the modified `CopyStatement`.
- [x] T004 [US1] Update the `test_parse_copy_statement` test in `src/parser/statements.rs` to test both single identifier (`COPY my_table`) and schema-qualified (`COPY cdm.concept`) syntax.
- [x] T005 [P] [US1] Update `src/executor/copy.rs` to handle the updated `CopyStatement`. Specifically, extract the `table_name.name.value()` and if present, the `table_name.schema.as_ref().map(|s| s.value())` to pass to the storage engine correctly.

## Phase 4: Polish & Cross-Cutting Concerns

- [x] T006 Ensure all code compiles and passes `make test` and `make lint`.