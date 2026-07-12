# Implementation Plan: Router Refinement & Standalone DAP TCP

**Branch**: `053-router-refinement-dap-tcp` | **Date**: Mon Jun 22 2026 | **Spec**: [spec.md](./spec.md)

## Summary

This feature addresses structural domain leaks, percent-decoding bugs, and query classification limitations inside Oxibase Server. It also implements a standalone TCP listener to make the embedded DAP (Debug Adapter Protocol) engine compatible with external standard IDE debuggers (e.g., VS Code, Neovim) outside the HTTP workspace browser environment.

## Technical Context

**Language/Version**: Rust 1.85+  
**Primary Dependencies**: axum, tokio (net, sync), serde_json, minijinja  
**Testing**: `cargo nextest run` (via `make test` / `make test-all`)  
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)  

## Constitution Check

- [x] **Mainframe Monolith**: Yes, DAP listeners and web routing continue to run locally within the monolithic database engine.
- [x] **ACID & MVCC**: Yes, any context query or RPC procedural executions follow strict MVCC isolation parameters.
- [x] **Memory Efficiency**: Yes, URL parsing and comment-stripping helpers avoid unnecessary string allocations.
- [x] **Safe Rust**: Yes, all panicking calls (`unwrap()`) are replaced with safe error propagation (`?` or `ok_or_else`). No new unsafe blocks.
- [x] **Tests First**: Yes, covered by integration tests in `tests/server_test.rs` and `tests/dap_test.rs`.

---

## Project Structure

```text
specs/053-router-refinement-dap-tcp/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Design decision log & alternatives
├── data-model.md        # Database-side models and RPC mappings
├── quickstart.md        # Quickstart setup and verification guide
└── contracts/
    └── routes.md        # TCP DAP Framing and RPC Payload contracts
```

---

## Phase 2 Implementation Steps

### Task 1: Refine `url_decode` to support Multi-Byte UTF-8
* **Location**: `src/server/handlers.rs`
* **Changes**: Rewrite `url_decode` to accumulate parsed hex percent characters into a `Vec<u8>` byte array, and then safely translate using `String::from_utf8_lossy(&bytes)`.

### Task 2: Robust SQL Query Prefix Classification
* **Location**: `src/server/handlers.rs`
* **Changes**: Write a helper `fn clean_sql_for_classification(sql: &str) -> String` that skips leading whitespaces and SQL comments (`--` and `/* ... */`). Use the cleaned string to accurately identify row-returning verbs (`SELECT`, etc.).

### Task 3: Shifting Workspace Domain Logic to RPC & Templates
* **Location**: `src/server/handlers.rs`
* **Changes**: Remove all path comparison block leaks (`if path == "/workspace/observe/logs"`, etc.) and manual pre-calculations from the Axum handler loop. 
* **Database / Seed**: Update the workspace templates in `src/bin/workspace/templates/` to use AJAX/Fetch requests querying database stored procedures (via the generic `/api/rpc/{proc}` endpoint) asynchronously to load UI metrics.

### Task 4: Standalone TCP DAP Listener & CLI Options
* **Location**: `src/server/dap.rs`, `src/bin/oxibase.rs`
* **Changes**: Add `--dap-tcp` and `--dap-port <port>` as global arguments under `Args` in `src/bin/oxibase.rs`. In `main()`, if `dap_tcp` is enabled, launch a background thread or tokio runtime task running the raw `TcpListener` loop. Stream framed packets from standard client connections directly to and from the `DebugController` state, enabling native external debugging in any serve, REPL, or script-execution modes concurrently.

### Task 5: Eliminate Handler `unwrap()`
* **Location**: `src/server/handlers.rs`
* **Changes**: Replace the RPC handler `unwrap()` on procedural results with robust, safe error mapping using `.ok_or_else()`.
