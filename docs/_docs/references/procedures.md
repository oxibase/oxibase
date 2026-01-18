---
layout: default
title: Stored Procedures
parent: References
nav_order: 6
---

# Stored Procedures

Stored procedures are blocks of code that can be executed on the database server. Unlike user-defined functions, procedures can:

- Execute arbitrary SQL statements using the `execute(sql)` function
- Access database context and metadata
- Perform data modifications (INSERT, UPDATE, DELETE)
- Run within the caller's transaction context
- Handle complex business logic with control flow (if/else, loops)

## Syntax

### Creating a Procedure

```sql
CREATE PROCEDURE procedure_name(parameter_list)
LANGUAGE backend AS 'procedure_body';
```

- `procedure_name`: The unique name of the procedure.
- `parameter_list`: A comma-separated list of parameters (e.g., `id INTEGER, name TEXT`).
- `backend`: The scripting language used (currently `rhai` is supported).
- `procedure_body`: The code string enclosed in single quotes.

**Note**: `CREATE ROUTINE` is also supported as an alias for `CREATE PROCEDURE`.

### Calling a Procedure

```sql
CALL procedure_name(argument_list);
```

- `procedure_name`: The name of the procedure to call.
- `argument_list`: The values to pass to the procedure.

### Dropping a Procedure

```sql
DROP PROCEDURE [IF EXISTS] procedure_name;
```

**Note**: `DROP ROUTINE` is also supported as an alias for `DROP PROCEDURE`.

### Showing Procedures

```sql
SHOW PROCEDURES;
```

**Note**: `SHOW ROUTINES` is also supported as an alias for `SHOW PROCEDURES`.

## Supported Languages

Currently, **Rhai** is the primary supported backend for stored procedures.

### Rhai Backend
Rhai is an embedded scripting language for Rust that provides a safe and easy way to write logic.
- **Syntax**: Similar to Rust and JavaScript.
- **Database Access**: Use the `execute(sql)` function to run SQL queries.
- **Return Values**:
  - For `SELECT` queries, `execute()` returns an array of row objects (maps).
  - For `INSERT/UPDATE/DELETE`, `execute()` returns the number of affected rows.

## Examples

### Basic Procedure

```sql
CREATE PROCEDURE greet_user(name TEXT)
LANGUAGE rhai AS '
    let message = "Hello, " + name + "!";
    print(message);
    
    // Log the greeting to a table
    execute(`INSERT INTO logs (message) VALUES ("${message}")`);
';
```

### Data Modification

```sql
CREATE PROCEDURE transfer_funds(from_id INTEGER, to_id INTEGER, amount FLOAT)
LANGUAGE rhai AS '
    // Check balance
    let rows = execute(`SELECT balance FROM accounts WHERE id = ${from_id}`);
    if rows.len() == 0 {
        throw "Source account not found";
    }
    
    let balance = rows[0].balance;
    if balance < amount {
        throw "Insufficient funds";
    }
    
    // Perform transfer
    execute(`UPDATE accounts SET balance = balance - ${amount} WHERE id = ${from_id}`);
    execute(`UPDATE accounts SET balance = balance + ${amount} WHERE id = ${to_id}`);
';
```

## Transaction Control

Procedures execute within the transaction context of the caller.
- If called inside a `BEGIN...COMMIT` block, the procedure's actions are part of that transaction.
- If an error occurs within the procedure (e.g., a thrown exception or SQL error), the transaction is aborted.

```sql
BEGIN;
CALL transfer_funds(1, 2, 100.0);
COMMIT;
```

## System Tables

Stored procedures are stored in the `_sys_procedures` system table. You can query this table to see defined procedures:

```sql
SELECT * FROM _sys_procedures;
```
