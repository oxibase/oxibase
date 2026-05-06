# Test Coverage Expansion Plan: Auto-API Layer

**Target File**: `src/server/handlers.rs`
**Current Coverage**: 57.58%

## 1. Goal
Achieve >85% patch coverage on `handlers.rs` by adding test cases for error conditions, edge cases, and JSON type conversions that are currently missing from the happy-path `test_auto_api_crud_flow` in `tests/server_test.rs`.

## 2. Missing Coverage Areas

### 2.1 Table Existence (404 and 500 Responses)
Currently, tests only operate on the `users` table which exists. We need tests that trigger:
- `404 Not Found`:
  - `GET /api/non_existent_table`
  - `POST /api/non_existent_table`
  - `PATCH /api/non_existent_table?id=eq.1`
  - `DELETE /api/non_existent_table?id=eq.1`

### 2.2 Malformed or Empty Payloads (400 Responses)
Currently, `POST` and `PATCH` use valid payloads. We need to trigger empty payload branches.
- `POST /api/users` with `{}` -> Expect 400 Bad Request
- `PATCH /api/users?id=eq.1` with `{}` -> Expect 400 Bad Request

### 2.3 Missing Filters on Mutating Requests (400 Responses)
Currently, `PATCH` and `DELETE` provide valid `?id=eq.1` filters. We need to test omission.
- `PATCH /api/users` with `{"name": "foo"}` -> Expect 400 Bad Request ("Missing exact match filter")
- `DELETE /api/users` -> Expect 400 Bad Request ("Missing exact match filter")

### 2.4 JSON Type Conversion Mapping
The current insert/update tests only send an integer (`1`) and a string (`"Alice"`). The `match v` block in `insert_row` and `update_row` handles multiple types. We need a test that pushes different JSON types through:
- `Null`
- `Bool(true/false)`
- `Number(float)`
- Array/Object (fallback serialization)
*Note: To test this properly, we need to setup a table with columns capable of holding these values (e.g., `CREATE TABLE complex_types (id INT, is_active BOOLEAN, score FLOAT, data JSON)`).*

### 2.5 Query Execution Errors (500 Responses)
We need to trigger SQL syntax or execution errors intentionally to cover the `Err(e)` branches of `state.db.query` and `state.db.execute`.
- **Insert Error**: Try inserting a string into an integer column (`"Alice"` into `id INT`).
- **Update Error**: Try updating an integer column with a string.
- **Select Error**: Use a bad `select` parameter (`?select=non_existent_column`).

## 3. Implementation Steps

We will add a new test function to `tests/server_test.rs`:

```rust
#[tokio::test]
async fn test_auto_api_edge_cases_and_errors() {
    let db = setup_db().await;
    // ... setup complex table ...
    let app = create_router(db);

    // Test 1: 404s for missing tables (GET, POST, PATCH, DELETE)
    // Test 2: 400s for empty payloads on POST/PATCH
    // Test 3: 400s for missing eq. filters on PATCH/DELETE
    // Test 4: JSON type conversions (inserting null, bool, float, array)
    // Test 5: 500s for DB execution errors (bad select column, type mismatch on insert)
}
```

This single new test function will comprehensively hit the remaining `Err` and `400`/`404` branches in `handlers.rs`, satisfying the Codecov check.
