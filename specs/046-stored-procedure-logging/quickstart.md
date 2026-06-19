# Quickstart Guide: Stored Procedure Logging

With this feature, direct `INSERT` statements on `system.logs` are blocked, and stored procedures in Rhai, Python, and PL/SQL have dedicated, safe, and highly performant logging features.

## 1. Logging in Rhai

Inside a Rhai-based stored procedure or function, call `oxibase::log`:

```sql
CREATE PROCEDURE test_rhai_logging()
LANGUAGE rhai
AS '
    oxibase::log("info", "Executing action in Rhai!");
    oxibase::log("warn", "Warning recorded!");
';
```

---

## 2. Logging in Python

Inside a Python-based stored procedure or function, use `oxibase.log`:

```sql
CREATE PROCEDURE test_python_logging()
LANGUAGE python
AS '
import oxibase

oxibase.log("info", "Python stored procedure starting")
oxibase.log("error", "An error has been tracked in python")
';
```

---

## 3. Logging in PL/SQL

In PL/SQL stored procedures, use the native `LOG` statement:

```sql
CREATE PROCEDURE test_plsql_logging(attempt_count INT)
LANGUAGE plsql
AS $$
BEGIN
    LOG INFO, 'PL/SQL log execution started';
    
    IF attempt_count > 3 THEN
        LOG WARN, 'Attempt count is high: ' || CAST(attempt_count AS TEXT);
    END IF;
END;
$$;
```
