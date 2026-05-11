# Tasks: Service Invocation via HTTP

**Input**: Design documents from `/specs/009-service-invocation/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature. 

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup & Foundational (Shared Infrastructure)

**Purpose**: Core logic updates required for HTTP invocation and header context.

- [x] T001 Define Thread-local `HTTP_HEADERS` context in `src/functions/context.rs` (or similar appropriate module).
- [x] T002 Implement `get_http_header` function in `src/functions/scalar/utility.rs`
- [x] T003 Register `get_http_header` in `src/functions/registry.rs`

---

## Phase 2: User Story 1 - Invoke a Stored Procedure via HTTP POST (Priority: P1) 🎯 MVP

**Goal**: Expose a generic `/api/rpc/:procedure_name` endpoint that handles JSON payload and executes the procedure correctly.

**Independent Test**: `tests/server_rpc_tests.rs` (Successful HTTP call)

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T010 [P] [US1] Create failing integration test in `tests/server_rpc_tests.rs` to create a sample procedure and invoke it over an active Axum server instance.

### Implementation for User Story 1

- [x] T011 [US1] Create `invoke_procedure` handler in `src/server/handlers.rs` to extract `procedure_name` and JSON body.
- [x] T012 [US1] Implement `get_procedure` lookup and argument validation (mapping JSON keys to argument indices) in `src/server/handlers.rs`.
- [x] T013 [US1] Call the DB `Executor` directly inside the handler (by formulating a `CALL` statement) and map output values back to a JSON response in `src/server/handlers.rs`.
- [x] T014 [US1] Wire up `POST /api/rpc/:procedure_name` in `src/server/mod.rs` to map to `invoke_procedure`.
- [x] T015 [US1] Update `tests/server_rpc_tests.rs` and verify passing output.

**Checkpoint**: User Story 1 should be fully functional; stored procedures can be invoked via HTTP successfully with 200 OK.

---

## Phase 3: User Story 2 - Handle Procedure Errors Gracefully (Priority: P1)

**Goal**: Make sure all exceptions and missing procedures map strictly to standard HTTP error codes.

**Independent Test**: `tests/server_rpc_tests.rs` (Error cases)

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T020 [P] [US2] Create failing integration tests in `tests/server_rpc_tests.rs` for 404 (not found), 400 (invalid param), and 500 (runtime error).

### Implementation for User Story 2

- [x] T021 [US2] Update `invoke_procedure` in `src/server/handlers.rs` to return HTTP 404 if `FunctionRegistry::get_procedure` returns `None`.
- [x] T022 [US2] Update parameter parsing in `src/server/handlers.rs` to return HTTP 400 with details if required parameters are missing or mistyped.
- [x] T023 [US2] Catch execution errors inside `invoke_procedure` and map to HTTP 500 with a clean JSON payload.
- [x] T024 [US2] Run `make test` to ensure error mapping works as expected.

**Checkpoint**: User Story 2 ensures the endpoint behaves robustly without crashing.

---

## Phase 4: User Story 3 - Pass Context Variables via HTTP Headers (Priority: P2)

**Goal**: Ensure HTTP headers sent in the POST request are captured and passed to the execution environment so `get_http_header` works correctly.

**Independent Test**: `tests/server_rpc_tests.rs` (Header context reading)

### Tests for User Story 3 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T030 [P] [US3] Create failing integration test in `tests/server_rpc_tests.rs` that defines a procedure invoking `get_http_header("Authorization")` and verifies the returned value.

### Implementation for User Story 3

- [x] T031 [US3] Update `invoke_procedure` handler in `src/server/handlers.rs` to extract HTTP headers from the request context.
- [x] T032 [US3] Wrap the execution call block in `src/server/handlers.rs` to set the `HTTP_HEADERS` thread-local context just before invocation and clear it after.
- [x] T033 [US3] Run `make test` to verify the procedure can access the passed headers correctly.

**Checkpoint**: Context variables are correctly propagated from the HTTP boundary to the script engine logic.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and ensure Constitution compliance.

- [x] T040 Verify `make license` passes (run `./scripts/fix_copyrights.sh` if needed).
- [x] T041 Verify `make lint` passes (`cargo fmt` and `cargo clippy -D warnings`).
- [x] T042 Ensure no `unwrap()` or `expect()` were introduced in the handler code.
- [x] T043 Code cleanup and refactoring in `src/server/handlers.rs` to keep the module manageable.