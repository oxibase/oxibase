# Phase 0: Research & Architecture Decisions

## 1. Syntax Validation at Creation
**Decision:** We will leverage the existing `ScriptingBackend::validate_code(&self, code: &str)` method.
**Rationale:** The `rhai` backend already implements this. During `CREATE PROCEDURE`, we will retrieve the requested language's backend from the `BackendRegistry` and validate the source code string. If syntax is invalid, it throws an error immediately, satisfying FR-010.

## 2. Parameter Modes (IN, OUT, INOUT)
**Decision:** Introduce `ParameterMode` enum in the AST and storage structs.
**Rationale:** Standard functions only use `IN` parameters and return a scalar value. Procedures return values by mutating `OUT` and `INOUT` parameters. We will map the outputs back into a single-row result set to return to the SQL client from the `CALL` statement.

## 3. Dedicated PL/SQL Interpreter
**Decision:** Build a custom AST and interpreter module under `src/functions/pl/sql/`.
**Rationale:** While we could theoretically lower PL/SQL directly into the main SQL executor, it's far easier to build a dedicated stack-based interpreter for variables and control flow. This fulfills FR-008 and critically supports FR-009 (DAP support) by isolating the execution state in an inspectable `Environment` frame.

## 4. System Catalog Separation
**Decision:** Create `system.procedures` table (`src/storage/procedures.rs`).
**Rationale:** Avoid polluting the existing `_sys_functions` table which expects strict scalar function signatures (`return_type`). Procedures have fundamentally different execution semantics and signatures.
