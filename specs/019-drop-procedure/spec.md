# Feature Specification: Drop Procedure

**Short Name**: drop-procedure
**Date**: 2026-05-16

## 1. Feature Description

Implementation of the `DROP PROCEDURE` SQL statement, allowing users to delete existing stored procedures from the database. Currently, users can create and call procedures but have no way to remove them without dropping and recreating the entire schema.

## 2. User Scenarios & Testing

### 2.1 Acceptance Scenarios
1. **Basic Deletion:**
   - User creates a stored procedure.
   - User issues `DROP PROCEDURE my_procedure`.
   - The procedure is successfully removed.
   - Subsequent calls to `my_procedure` result in an error indicating the procedure does not exist.

2. **Deletion with IF EXISTS:**
   - User issues `DROP PROCEDURE IF EXISTS non_existent_procedure`.
   - The statement executes successfully without returning an error.
   - User issues `DROP PROCEDURE IF EXISTS existing_procedure`.
   - The procedure is successfully removed without returning an error.

### 2.2 Error and Edge Cases
1. **Dropping Non-Existent Procedure (without IF EXISTS):**
   - User issues `DROP PROCEDURE non_existent_procedure`.
   - The system returns a clear error indicating the procedure was not found.

## 3. Functional Requirements

1. The SQL parser must recognize the `DROP PROCEDURE` syntax.
2. The SQL parser must optionally accept `IF EXISTS` within the `DROP PROCEDURE` statement.
3. The SQL abstract syntax tree (AST) must represent the `DROP PROCEDURE` statement.
4. The execution engine must handle the parsed `DROP PROCEDURE` statement and remove the specified procedure from the database catalog/storage.
5. Dropping a procedure that does not exist without the `IF EXISTS` clause must return a standard "object not found" error.
6. Using `IF EXISTS` when the procedure does not exist must bypass the error and execute successfully.

## 4. Success Criteria

1. Users can successfully delete procedures using standard SQL syntax.
2. Attempting to call a deleted procedure fails as expected.
3. Integration tests verify that `CREATE`, `CALL`, and `DROP PROCEDURE` operations work together harmoniously.

## 5. Key Entities

- **Procedure:** The executable routine stored in the database.
- **Statement:** The `DROP PROCEDURE` SQL command.

## 6. Assumptions

- Procedures do not currently support complex dependency tracking (e.g., if a view or another procedure depends on a procedure being dropped, the drop will still proceed). 
- Procedure dropping uses standard permission checks existing in the execution engine.
