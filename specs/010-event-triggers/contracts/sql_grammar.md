# Interface Contract: Trigger SQL Grammar

This document defines the new SQL syntax exposed to end-users for managing triggers.

## `CREATE TRIGGER`

Creates a new trigger associated with a specific table.

```sql
CREATE TRIGGER [IF NOT EXISTS] trigger_name
    { BEFORE | AFTER } { INSERT | UPDATE | DELETE }
    ON table_name
    [ FOR EACH ROW ]
    LANGUAGE { rhai | js | python }
AS $$
    -- procedural body referencing NEW and OLD
$$;
```

**Variables Available in Procedural Body:**
*   `NEW`: A proxy object representing the new row state. Available in `INSERT` and `UPDATE` triggers.
*   `OLD`: A proxy object representing the existing row state. Available in `UPDATE` and `DELETE` triggers.

## `DROP TRIGGER`

Removes an existing trigger.

```sql
DROP TRIGGER [IF EXISTS] trigger_name ON table_name;
```
