# Tasks: Router Refinement & Standalone DAP TCP

This document contains the step-by-step task checklist required to complete the router refactoring and standalone TCP DAP integration.

## Implementation Strategy

We follow an incremental delivery approach, completing and verifying each user story phase independently. Core bug-fixes (URL decoding, SQL classification) are prioritized as foundational MVP tasks, followed by telemetry RPC conversion and standalone TCP listener support.

---

## Task Checklist

### Phase 1: Setup

Goal: Bootstrap the database routines and prepare workspace files.

- [x] T001 Define stored procedures `system.get_log_histogram` and `system.get_trace_timeline` inside the database bootstrap routines in `src/bin/workspace/mod.rs`

---

### Phase 2: Foundational

Goal: Solve query execution safety and core classification utilities.

- [x] T002 Implement a lexical SQL comment-stripping and whitespace-cleaning helper function `clean_sql_for_classification` in `src/server/handlers.rs`
- [x] T003 Replace the direct RPC result map `.unwrap()` call in `src/server/handlers.rs` line 1237 with robust, safe error propagation

---

### Phase 3: User Story 1 - Multi-Byte URL Filter Querying (P1)

Goal: Fix percent URL decoding to prevent data corruption on non-ASCII characters.

* **Independent Test Criteria**: A GET request containing percent-encoded multi-byte UTF-8 parameters matches the database query records successfully.

- [x] T004 [P] [US1] Refactor `url_decode` in `src/server/handlers.rs` to collect percent-encoded bytes inside a binary `Vec<u8>` buffer and translate to string at the end
- [x] T005 [US1] Write unit tests inside `tests/server_test.rs` to verify correct multi-byte character query filtering and matching

---

### Phase 4: User Story 2 - SQL Query Classification (P1)

Goal: Correctly execute custom formatted queries documented with leading comments or whitespaces.

* **Independent Test Criteria**: Posting a query with leading single/multi-line SQL comments and newlines returns the query results instead of an empty action block.

- [x] T006 [P] [US2] Integrate the `clean_sql_for_classification` helper into query checks in `src/server/handlers.rs` to ensure correct read-vs-write classification
- [x] T007 [US2] Write integration tests inside `tests/server_test.rs` verifying that SELECT queries with leading indentation and comments return output rows

---

### Phase 5: User Story 3 - Clean Workspace Domain Separation (P2)

Goal: Eliminate all hardcoded telemetry path checks and pre-processing from the Rust handlers, shifting them to database views and async RPC calls.

* **Independent Test Criteria**: Complete removal of workspace paths comparison block from handlers, with zero regressions on workspace observability dashboards.

- [x] T008 [P] [US3] Remove hardcoded paths checks (`/workspace/observe/logs`, `/workspace/observe/traces`, and `/workspace/run_modal`) and telemetry pre-formatting loops from `src/server/handlers.rs`
- [x] T009 [US3] Refactor `src/bin/workspace/templates/workspace_observe_logs.html` to load the logs histogram asynchronously from `system.get_log_histogram` via dynamic AJAX Fetch calls
- [x] T010 [US3] Refactor `src/bin/workspace/templates/workspace_trace_view.html` to load trace spans and bounds asynchronously from `system.get_trace_timeline` via dynamic AJAX Fetch calls

---

### Phase 6: User Story 4 - Standalone TCP DAP Listener (P1)

Goal: Bridge external DAP debuggers (VS Code, Neovim) directly to the database interpreter over raw TCP.

* **Independent Test Criteria**: A standard external TCP DAP client connects to the specified port, performs the initialize handshake, sets a breakpoint, and receives stopped events.

- [x] T011 [P] [US4] Add global CLI options `--dap-tcp` (boolean) and `--dap-port` (u16, default: 4711) under `Args` struct in `src/bin/oxibase.rs`
- [x] T012 [P] [US4] Implement standard raw TCP stream decoder and framer processing `Content-Length` headers in `src/server/dap.rs`
- [x] T013 [US4] Update CLI entrypoint in `src/bin/oxibase.rs` to spawn the TCP DAP listener loop concurrently inside `main` if the `--dap-tcp` flag is enabled
- [x] T014 [US4] Write an integration test suite inside `tests/server_test.rs` simulating an external DAP client session connecting, handshaking, and setting breakpoints

---

### Phase 7: Polish & Cross-Cutting Concerns

Goal: Ensure lints, copyrights, and test coverage standards are fully satisfied.

- [x] T015 Run `make lint` to verify clean formatting and check for clippy violations across all targets
- [x] T016 Run `./scripts/fix_copyrights.sh` or verify correct Apache-2.0 license headers on any modified files
- [x] T017 Run `make test-all` to ensure entire system test suite passes successfully

---

## Dependencies & Completion Order

```text
Phase 1: Setup -> Phase 2: Foundational -> Phase 3: User Story 1 (P1)
                                        -> Phase 4: User Story 2 (P1)
                                        -> Phase 5: User Story 3 (P2)
                                        -> Phase 6: User Story 4 (P1) -> Phase 7: Polish
```

## Parallel Execution Examples

* **Example 1**: Task `T004` (Refactoring URL decoder in server handlers) and Task `T011`/`T012` (CLI Args and TCP framing in `oxibase.rs` / `dap.rs`) have zero mutual file locks and can be developed concurrently.
* **Example 2**: Task `T006` (Query classification in server handlers) and Task `T008` (Removing hardcoded path checks) can be implemented in parallel.
