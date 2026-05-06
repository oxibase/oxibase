# Feature Specification: Fix Transaction Updates with Foreign Keys

**Feature Branch**: `002-fix-tx-update-fk`  
**Created**: 2026-05-05  
**Status**: Draft  
**Input**: User description: "when executing the oxibase cli i get some error in this sequence of steps... Error: internal error: row 2 has uncommitted changes from transaction 9... Error: internal error: row 3 has uncommitted changes from transaction 9"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Commit Updates with Foreign Keys in Transactions (Priority: P1)

Users must be able to successfully update rows that are referenced by foreign keys within a transaction, and commit those changes without internal MVCC errors.

**Why this priority**: Transactional integrity is a core database feature. Internal errors during normal SQL operations (like updates within a transaction) indicate a severe bug that breaks basic functionality and trust.

**Independent Test**: Can be fully tested by a new integration test mimicking the user's script: creating tables with an FK constraint, starting a transaction, updating referenced rows, and committing, verifying it succeeds without error.

**Acceptance Scenarios**:

1. **Given** a `categories` table and a `products` table with a foreign key referencing `categories`, **When** a user starts a transaction, updates multiple `products` rows, and attempts to commit, **Then** the transaction should commit successfully, returning the updated rows upon query.
2. **Given** a transaction modifying rows, **When** those rows are referenced by a newly added foreign key constraint in the same or subsequent transaction, **Then** updates should process correctly without MVCC conflicts against itself.

---

### User Story 2 - Rollback Updates with Foreign Keys in Transactions (Priority: P2)

Users must be able to successfully roll back updates to rows that are referenced by foreign keys, restoring the state prior to the transaction without internal MVCC errors.

**Why this priority**: Rollback functionality is just as critical as commit functionality for ACID compliance. The reported script also shows a rollback scenario before the error.

**Independent Test**: Can be fully tested by a new integration test that starts a transaction, updates referenced rows, rolls back, and verifies the original state is restored without errors.

**Acceptance Scenarios**:

1. **Given** a transaction updating referenced rows, **When** the user issues a `ROLLBACK`, **Then** the transaction should abort, and the rows should revert to their pre-transaction state without throwing "uncommitted changes" errors on subsequent operations.

### Edge Cases

- What happens when concurrent transactions attempt to update the same rows referenced by foreign keys?
- How does the system handle an `ALTER TABLE ADD CONSTRAINT` operation immediately following transactions that updated the referenced data?
- How does MVCC track active transaction IDs for rows involved in referential integrity checks?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Engine MUST correctly manage MVCC row versions during `UPDATE` operations within a transaction so that subsequent operations or commits recognize the transaction's own changes.
- **FR-002**: Storage MUST NOT incorrectly flag rows modified by the *current* transaction as having "uncommitted changes" from a conflicting transaction when performing foreign key checks or final commits.
- **FR-003**: System MUST correctly handle the interaction between transaction state and newly applied schema constraints (like `ALTER TABLE ADD CONSTRAINT`).

### Key Entities

- **Transaction Manager**: Tracks active transactions and their modifications.
- **MVCC Versioning**: Manages visibility and uncommitted changes for rows.
- **Foreign Key Checker**: Validates referential integrity, which must interact correctly with MVCC visibility rules for the active transaction.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The exact SQL script provided by the user executes successfully without throwing "Error: internal error: row X has uncommitted changes from transaction Y".
- **SC-002**: Passes all new integration tests covering transactions, updates, and foreign key constraints.
- **SC-003**: Passes `make lint` and `cargo nextest` without regressions in existing functionality.

## Assumptions

- The issue stems from the interaction between how `UPDATE` modifies row versions and how the transaction manager tracks uncommitted changes, specifically exposed when foreign key constraints necessitate row lookups or validations.
- The user's provided SQL script is standard SQL and should be fully supported by Oxibase.
