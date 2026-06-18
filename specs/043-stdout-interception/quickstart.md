# Quickstart: Scripting Stdout Interception

This guide demonstrates how to use `print()` in Rhai and `PRINT` / `RAISE NOTICE` in PL/SQL to output debug information during procedure execution.

## Rhai Execution Output

You can use standard `print()` inside your Rhai scripts.

```sql
CREATE OR REPLACE PROCEDURE debug_rhai() LANGUAGE rhai AS $$
    print("Execution started...");
    let x = 1 + 2;
    print("x is now " + x);
$$;

CALL debug_rhai();
```
*When executed, the execution context logs will contain:*
```text
Execution started...
x is now 3
```

## PL/SQL Execution Output

You can use `PRINT` or `RAISE NOTICE` within PL/SQL blocks.

```sql
CREATE OR REPLACE PROCEDURE debug_plsql() LANGUAGE plsql AS $$
DECLARE
    counter INT := 0;
BEGIN
    PRINT 'Starting procedure...';
    counter := counter + 5;
    RAISE NOTICE counter;
END;
$$;

CALL debug_plsql();
```
*When executed, the execution context logs will contain:*
```text
Starting procedure...
5
```