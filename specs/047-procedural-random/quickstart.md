# Quickstart: Procedural Random Number Generation

This guide shows you how to use native random number generation in stored procedures and scripts inside Oxibase.

## Rhai
In a Rhai procedure, call `oxibase::random()`:

```sql
CREATE PROCEDURE test_rhai_random(OUT val FLOAT)
LANGUAGE rhai AS '
    val = oxibase::random();
';
```

---

## Python
In a Python procedure, import `oxibase` and call `oxibase.random()`:

```sql
CREATE PROCEDURE test_python_random(OUT val FLOAT)
LANGUAGE python AS '
import oxibase
val = oxibase.random()
';
```

---

## PL/SQL
In a PL/SQL procedure or function, call `random()` directly:

```sql
CREATE FUNCTION test_plsql_random() RETURNS FLOAT
LANGUAGE plsql AS '
DECLARE
    r FLOAT;
BEGIN
    r := random();
    RETURN r;
END;
';
```
