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
