# Phase 0: Research Findings

## Topic 1: Procedural Engine Instantiation
**Decision**: Leverage the existing backend registration pipeline but ensure trigger caching respects engine lifecycles.
**Rationale**: `oxibase` uses a hybrid approach. Rhai uses a single global `Engine` with per-execution isolated `Scope`. Boa (JS) and RustPython create a fresh `Context` or `Interpreter` per execution. The backends are registered via a global `OnceLock` (`GLOBAL_REGISTRY`). Triggers will retrieve the backend from this registry upon execution.
**Alternative**: Creating an engine per row would be prohibitively expensive for Boa/Python. The current architecture already handles strict isolation.

## Topic 2: AST Representation and Metadata Cataloging
**Decision**: Use `CreateTriggerStatement` AST node and a `_sys_triggers` catalog table with an in-memory `TriggerRegistry`.
**Rationale**: Mirroring existing DDL (like `CreateProcedureStatement`), triggers will be parsed into an AST block containing timing, event, table, language, and script body. They will be stored in `_sys_triggers` system table. To avoid catastrophic performance degradation during row-by-row DML pipelines (e.g., inside `executor/dml.rs`), the Executor must maintain a `TriggerRegistry` cache initialized from `_sys_triggers` on boot, allowing zero-overhead O(1) lookups.
**Alternative**: Hiding trigger data in obscure files or fetching from disk per DML operation. Both violate architecture or performance constraints.

## Topic 3: Zero-Copy Row Passing via Proxy Pattern
**Decision**: Implement lazy-evaluation using a safe Thread-Local Proxy Pattern.
**Rationale**: Eagerly converting `&mut [Value]` to native script types (like `rhai::Dynamic`) for every column in a row, for every row inserted, will cause a massive memory allocation bottleneck. Instead, we use thread-local pointers (`CURRENT_NEW_ROW`, `CURRENT_OLD_ROW`, `CURRENT_SCHEMA`) safely cleared after script execution. We inject empty Proxy objects (via `register_getter_fallback` in Rhai, JS Proxy in Boa, `__getattr__` in Python) into the script scope. These proxies lazily convert ONLY the specific `Value` requested by the trigger logic at runtime.
**Alternative**: Deep cloning the `Row` to avoid lifetime constraints, which violates zero-copy goals and would fail performance requirements.

## Topic 4: Edge Cases & Error Handling
**Decision**: 
1. **Recursion Limit**: A thread-local `TRIGGER_DEPTH_COUNTER: Cell<usize>` will be implemented. It increments before executing a script and decrements after. If it exceeds `MAX_TRIGGER_DEPTH` (default: 32), the engine aborts the transaction to prevent stack overflows.
2. **Missing State**: `OLD` row is represented as `None` (injected as `null`/`None` proxy in scripts) during `INSERT`. `NEW` row is `None` during `DELETE`.
3. **Exceptions**: Unhandled script exceptions are intercepted by `execute_procedure` and bubbled up as `DbError::TriggerExecutionError`, ensuring the parent DML loop (`src/executor/dml.rs`) rolls back the transaction.
4. **Schema Drops**: When `DROP TABLE` is executed in `src/executor/ddl.rs`, a cascading step iterates `_sys_triggers` and removes all associated triggers from both storage and the in-memory `TriggerRegistry`.

## Topic 5: Performance Measurement
**Decision**: 
1. **Zero-Copy Check**: The trigger context `with_trigger_context` takes a raw pointer (`*mut Row` and `*const Row`) to avoid `.clone()` entirely. The proxies lazily access the pointer.
2. **Benchmarking Methodology**: We will run standard `INSERT`/`UPDATE` benchmark suites on a table with 0 triggers, measure TPS (Transactions Per Second), and assert that the addition of the empty `TriggerRegistry` lookup in `executor/dml.rs` results in `< 5%` drop in TPS.
