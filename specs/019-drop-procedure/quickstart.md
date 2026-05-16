# Quickstart: Drop Procedure

This feature allows users to drop a stored procedure using standard SQL.

## Usage Examples

**Basic drop:**

```sql
CREATE PROCEDURE my_proc() LANGUAGE sql AS $$ SELECT 1; $$;

-- Drop the procedure
DROP PROCEDURE my_proc;
```

**Drop with IF EXISTS (idempotent):**

```sql
-- Will not error even if `my_proc` doesn't exist
DROP PROCEDURE IF EXISTS my_proc;
```
