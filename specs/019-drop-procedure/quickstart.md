# Quickstart: Drop Procedure

## Usage Example

```sql
-- Create a procedure first
CREATE PROCEDURE my_procedure (param1 INT)
LANGUAGE SQL
AS $$
    -- Procedure logic
$$;

-- Drop the procedure
DROP PROCEDURE my_procedure;

-- Drop with IF EXISTS to avoid errors if it's already deleted
DROP PROCEDURE IF EXISTS my_procedure;
```
