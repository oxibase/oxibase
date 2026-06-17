---
layout: default
title: Stored Procedures
parent: References
nav_order: 6
---

# Stored Procedures

Stored procedures are blocks of code that can be executed on the database server. Unlike user-defined functions, procedures can:

- Execute multiple SQL statements
- Access database context and metadata
- Perform data modifications (INSERT, UPDATE, DELETE)
- Control transaction flow implicitly or through SQL bindings
- Mutate `OUT` and `INOUT` parameters to return results

## Syntax

```sql
CREATE [OR REPLACE] PROCEDURE procedure_name(parameter_list)
LANGUAGE backend AS $$
    -- procedure body
$$;
```

## Parameters

Procedures support the following parameter modes:
- **`IN`** (Default): The parameter is passed into the procedure but cannot be passed back to the caller.
- **`OUT`**: The procedure assigns a value to this parameter, returning it to the caller as part of a result set.
- **`INOUT`**: The parameter provides an initial value, and the procedure can mutate it to return the new value.

## Supported Backends

- **`rhai`**: Lightweight scripting with access to database context (Default)
- **`python`**: Python scripting environment (Requires `--features python`)
- **`plsql`**: Native database procedural language clone (Always available)

## Examples

### Rhai Procedure

```sql
CREATE OR REPLACE PROCEDURE update_inventory(IN product_id INTEGER, IN quantity_change INTEGER, OUT success BOOLEAN)
LANGUAGE rhai AS $$
    // Execute DML and get affected rows
    let query = "UPDATE inventory SET quantity = quantity + " + to_string(quantity_change) + " WHERE id = " + to_string(product_id);
    let rows_affected = oxibase::execute(query);
    
    if rows_affected > 0 {
        success = true;
    } else {
        success = false;
    }
$$;
```

### Calling Procedures

```sql
-- Execute a procedure
CALL update_inventory(123, -5, false);

-- Output returns a single row mapping the mutated OUT/INOUT parameters
```

## Differences from Functions

| Aspect | Functions | Procedures |
|--------|-----------|------------|
| Return Values | Always return a single value (`RETURNS type`) | Return state mutations via `OUT`/`INOUT` parameters |
| SQL Usage | Can be used in SELECT, WHERE, etc. | Called standalone with `CALL` statement |
| Side Effects | Pure functions (no side effects) | Can modify data and database state |
| Context Access | Limited to input parameters | Full access to database context via `oxibase` / `oxibase::execute` bindings |

## Transaction Control

Stored Procedures run within the transactional context of the executor. If an unhandled exception or error occurs during execution, the active transaction will safely abort and roll back.

You can explicitly control transaction boundaries inside stored procedures. This is particularly useful for long-running batch operations where you want to commit chunks of work to avoid large rollbacks.

> **Note**: Transaction control inside procedures is only permitted if the `CALL` is not inside an explicit transaction block (i.e., you haven't run `BEGIN;` before the `CALL`). Otherwise, it will throw an "invalid transaction termination" error.

### Language Syntax

- **PL/SQL**: Use native `COMMIT;`, `ROLLBACK;`, and `BEGIN;` statements.
- **Rhai**: Use `oxibase::commit()`, `oxibase::rollback()`, and `oxibase::begin()`.
- **Python**: Use `oxibase.commit()`, `oxibase.rollback()`, and `oxibase.begin()`.

### Example (PL/SQL)

```sql
CREATE PROCEDURE process_batch()
LANGUAGE plsql AS $$
BEGIN
    UPDATE jobs SET status = 'processing' WHERE id = 1;
    COMMIT; -- Commits the active transaction and immediately starts a new one
    
    UPDATE jobs SET status = 'done' WHERE id = 1;
    COMMIT;
END;
$$;
```
