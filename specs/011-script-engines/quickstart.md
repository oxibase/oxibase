# Quickstart: Python & JS Triggers

This guide outlines how to use the optional scripting engines (`python` and `js`) to write triggers. Ensure Oxibase is compiled with `--features python,js`.

## Python Validation Trigger

Abort an insert if the balance is negative.

```sql
CREATE TRIGGER validate_python
    BEFORE INSERT ON accounts
    FOR EACH ROW
    LANGUAGE python
AS $$
    if NEW.balance < 0:
        raise RuntimeError("Balance cannot be negative")
$$;
```

## Python Data Transformation

Prefix a string before saving.

```sql
CREATE TRIGGER transform_python
    BEFORE UPDATE ON users
    FOR EACH ROW
    LANGUAGE python
AS $$
    NEW.name = "PY_PREFIX_" + NEW.name
$$;
```

## JavaScript Audit Trigger

Log changes to an audit table after an update.

```sql
CREATE TRIGGER audit_js
    AFTER UPDATE ON accounts
    FOR EACH ROW
    LANGUAGE js
AS $$
    if (OLD.balance !== NEW.balance) {
        let stmt = "INSERT INTO audit_log (account_id, old_balance, new_balance) VALUES (" 
                   + OLD.id + ", " + OLD.balance + ", " + NEW.balance + ")";
        oxibase.execute(stmt);
    }
$$;
```
