# Implementation Plan: Service Invocation via HTTP

**Branch**: `009-service-invocation` | **Date**: 2026-05-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/009-service-invocation/spec.md`

## Summary

Expose stored procedures over HTTP via a new `POST /api/rpc/:procedure_name` route. The handler will accept JSON payloads, map them to procedure arguments, and invoke the procedure within the database engine using `CALL`. It also introduces a `get_http_header` built-in SQL function to let procedures access HTTP request metadata via a thread-local context.

## Technical Context

**Language/Version**: Rust 1.85+ 
**Primary Dependencies**: axum, serde_json, tokio
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Passed*

- [x] **Mainframe Monolith**: Yes. Keeps logic inside the database by exposing it directly via the built-in HTTP server.
- [x] **ACID & MVCC**: Yes. `CALL` handles implicit transactions properly.
- [x] **Memory Efficiency**: Yes. Uses Axum's standard JSON extractors and avoids excessive copying.
- [x] **Safe Rust**: Yes. Proper mapping of HTTP errors without panics.
- [x] **Tests First**: Yes. Integration tests will be added for the HTTP endpoints.

## Project Structure

### Documentation

```text
specs/009-service-invocation/
├── plan.md              # This file 
├── research.md          # Architectural decisions
├── data-model.md        # API contracts
└── spec.md              # Feature requirements
```

### Source Code

```text
src/
├── server/mod.rs                 # Add /api/rpc/:procedure route
├── server/handlers.rs            # Implement invoke_procedure handler and header context wrapper
├── functions/context.rs          # (New) Thread-local for HTTP headers
├── functions/scalar/utility.rs   # Add get_http_header built-in function
├── functions/registry.rs         # Register get_http_header
└── tests/server_rpc_tests.rs     # Integration tests for the new endpoint
```

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |