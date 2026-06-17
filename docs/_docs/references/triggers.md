---
layout: default
title: Triggers
parent: References
nav_order: 7
---

# Event Triggers

Triggers are database objects that automatically execute procedural logic when specified data manipulation events (`INSERT`, `UPDATE`, or `DELETE`) occur on a table. 

Oxibase supports **row-level** event triggers (`FOR EACH ROW`), meaning the trigger logic is executed once for every row affected by the statement. This allows developers to validate data, transform inputs, or perform side-effects like audit logging entirely within the database engine.

## Syntax

### CREATE TRIGGER

```sql
CREATE TRIGGER [IF NOT EXISTS] trigger_name
    { BEFORE | AFTER } { INSERT | UPDATE | DELETE }
    ON table_name
    [ FOR EACH ROW ]
    LANGUAGE { rhai | js | python }
AS $$
    -- Procedural logic goes here
$$;
```

*Note: The script body can be enclosed in `$$ ... $$` or standard single quotes `' ... '`.*

### DROP TRIGGER

```sql
DROP TRIGGER [IF EXISTS] trigger_name ON table_name;
```

## Supported Languages

Triggers can be written in any scripting backend supported by your Oxibase installation:
- **`rhai`**: The default embedded scripting language (always available).
- **`plsql`**: The built-in procedural SQL language (always available).
- **`js`**: JavaScript via Boa Engine (requires compiling with `--features js`).
- **`python`**: Python via RustPython (requires compiling with `--features python`).

## Execution Timing (`BEFORE` vs `AFTER`)

- **`BEFORE` Triggers**: Execute *before* the row is persisted to the storage engine. They are typically used for data validation or data transformation. Modifying the `oxibase.ctx.new` row inside a `BEFORE` trigger will alter the data that is ultimately saved.
- **`AFTER` Triggers**: Execute *after* the row is successfully persisted but before the transaction completes. They are typically used for side-effects, such as logging to an audit table.

## Row Context (`oxibase.ctx.new` and `oxibase.ctx.old`)

Inside the procedural trigger body, the engine exposes proxy objects representing the row being modified under the `oxibase.ctx` namespace. Because Oxibase uses a zero-copy proxy pattern, these objects do not clone the underlying data, making them highly efficient.

| Event | `oxibase.ctx.old` object | `oxibase.ctx.new` object |
| :--- | :--- | :--- |
| **`INSERT`** | `null` / `None` | Contains the new values being inserted. (Writable in `BEFORE` triggers) |
| **`UPDATE`** | Contains the original values before modification. (Read-only) | Contains the new values. (Writable in `BEFORE` triggers) |
| **`DELETE`** | Contains the values of the row being deleted. (Read-only) | `null` / `None` |

### Accessing Columns

You access row data via property/attribute access mapping exactly to your table schema.

- **Rhai**: `oxibase.ctx.new.column_name`
- **PL/SQL**: `NEW.column_name` and `OLD.column_name`
- **JavaScript**: `oxibase.ctx.new.column_name`
- **Python**: `oxibase.ctx.new.column_name` (or `oxibase.ctx.new['column_name']` depending on dictionary implementation)

## Error Handling and Transaction Aborts

Triggers execute within the same transaction context as the statement that fired them. If a trigger encounters an error or explicitly throws an exception, the entire statement is safely rolled back.

- **Rhai**: `throw "Invalid data";`
- **PL/SQL**: N/A (Standard `RAISE` not yet implemented in MVP)
- **JavaScript**: `throw new Error("Invalid data");`
- **Python**: `raise RuntimeError("Invalid data")`

## Executing SQL inside Triggers

Triggers can perform side-effects by executing other SQL statements (e.g., inserting into an audit log). The exact syntax depends on the language:

- **Rhai**: `oxibase::execute("INSERT INTO log (msg) VALUES ('test')");`
- **PL/SQL**: Direct SQL execution: `INSERT INTO log (msg) VALUES ('test');`
- **JavaScript**: `oxibase.execute("INSERT INTO log (msg) VALUES ('test')");`
- **Python**: `oxibase.execute("INSERT INTO log (msg) VALUES ('test')")`

## Edge Cases

- **Recursion Protection**: To prevent infinite loops (e.g., an `AFTER INSERT` trigger on Table A inserting a row into Table A), Oxibase limits trigger execution depth to `32` nested calls. Exceeding this will abort the transaction.
- **Cascading Drops**: When a table is dropped via `DROP TABLE`, all associated triggers are automatically removed from the system catalog.
