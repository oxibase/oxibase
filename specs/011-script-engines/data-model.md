# Phase 1: Data Model

*(No new entities or catalog changes are introduced. This feature builds entirely upon the AST and Storage elements defined in `010-event-triggers`.)*

## In-Memory Proxy Entities

### Python Proxies (`oxibase_py_module`)
*   `PyNewRowProxy`: A `#[pyclass]` injected globally as `NEW`. Implements `getattr` and `setattr` magic methods.
*   `PyOldRowProxy`: A `#[pyclass]` injected globally as `OLD`. Implements `getattr` only (read-only).

### JavaScript Proxies (`boa_engine`)
*   `JsProxy (NEW)`: Built via `JsProxy::builder` with native Rust `get` and `set` traps. Injected into the global `Context` as `NEW`.
*   `JsProxy (OLD)`: Built via `JsProxy::builder` with a native Rust `get` trap only. Injected into the global `Context` as `OLD`.

Both systems resolve dynamic property accesses by communicating with the thread locals (`CURRENT_NEW_ROW`, `CURRENT_OLD_ROW`, `CURRENT_SCHEMA`) managed in `src/functions/backends/triggers.rs`.
