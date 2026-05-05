---
description: "Task list for fixing the transaction updates with foreign keys bug"
---

# Tasks: Fix Transaction Updates with Foreign Keys

**Input**: Design documents from `/specs/002-fix-tx-update-fk/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md

**Tests**: MUST include corresponding `cargo nextest` integration tests.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- `src/storage/mvcc/` for MVCC and engine state changes
- `tests/` for integration tests

---

## Phase 1: Setup

**Purpose**: Project initialization and basic structure

*(No general setup tasks needed, as the project is already initialized and the fix operates on existing files)*

---

## Phase 2: Foundational Changes

**Purpose**: Shared infrastructure and trait changes required by all user stories.

- [X] T001 Define `rollback_all_tables(&self, txn_id: i64) -> Result<()>` in the `TransactionEngineOperations` trait in `src/storage/mvcc/transaction.rs`.

---

## Phase 3: User Story 2 - Rollback Updates with Foreign Keys in Transactions (Priority: P2)

**Goal**: Users must be able to successfully roll back updates to rows that are referenced by foreign keys, restoring the state prior to the transaction without internal MVCC errors.
*Note: Addressed before US1 because the specific engine leak (cache not cleared on rollback) is the root cause preventing subsequent operations (like adding FKs and committing).*

**Independent Test**: `tests/transaction_rollback_fk_test.rs` (or similar existing transaction test file)

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T002 [US2] Create or update an integration test in `tests/transaction_fk.rs` that starts a transaction, updates rows referenced by foreign keys, executes `ROLLBACK`, and verifies that subsequent operations (like `ALTER TABLE ADD CONSTRAINT` or new updates) succeed.

### Implementation for User Story 2

- [X] T003 [P] [US2] Implement `rollback_all_tables` in `MVCCEngine` within `src/storage/mvcc/engine.rs` to iterate over `txn_version_stores`, call `rollback()` on matching stores, and then `retain` only those stores belonging to other transactions.
- [X] T004 [US2] Update `MvccTransaction::rollback` in `src/storage/mvcc/transaction.rs` to call `ops.rollback_all_tables(self.id)` instead of the current incomplete loop over `self.tables`. Remove the redundant calls to `ops.rollback_table` as they will be handled by the new `rollback_all_tables` function.
- [X] T005 [P] [US2] Implement the `Drop` trait for `TransactionVersionStore` in `src/storage/mvcc/version_store.rs` to automatically call `self.release_all_claims()` as a defense-in-depth measure against panics or dropped transactions.
- [X] T006 [US2] Run `cargo nextest run` to verify the new integration test passes and the rollback behavior successfully clears row claims.

---

## Phase 4: User Story 1 - Commit Updates with Foreign Keys in Transactions (Priority: P1) 🎯 MVP

**Goal**: Users must be able to successfully update rows that are referenced by foreign keys within a transaction, and commit those changes without internal MVCC errors.
*Note: With the rollback leak fixed and the engine cache management unified, the commit path and subsequent constraint additions will succeed.*

**Independent Test**: Included in the same test file.

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T007 [US1] Add a test case in `tests/transaction_fk.rs` that updates rows referenced by foreign keys inside a transaction, commits the transaction, and verifies the update persists and constraints are respected without MVCC internal errors.

### Implementation for User Story 1

- [X] T008 [US1] The foundational changes in Phase 2 & 3 address the caching architecture flaw that also affects sequential transaction integrity. Verify that no further `VersionStore` or `MvccEngine` changes are required for the commit path. 
- [X] T009 [US1] Run `make test` and `make lint` to ensure no regressions and that the complete user script (as reported) executes flawlessly.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T010 [P] Verify `make license` passes to ensure new test files have the correct Apache-2.0 header.
- [X] T011 Verify `unwrap()` and `expect()` are not used inappropriately in the new `engine.rs` or `version_store.rs` code (handle lock acquisition properly if needed, though `.unwrap()` on `RwLock` is standard in this codebase for panic-on-poisoning).
