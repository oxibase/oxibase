# Quickstart & Verification: plsql-table-iteration

## Quickstart

Once the feature is fully implemented, you can verify it by creating and running the user-defined functions on the running `oxibase` server at `localhost:8080`.

---

## Step 1: Verify Rhai Backend Integration

Create the Rhai query function:
```sql
CREATE FUNCTION test_query_rhai() RETURNS INT
LANGUAGE RHAI AS '
    let rows = oxibase::query("SELECT 100 as val;");
    let val = rows[0]["val"];
    return val;
';
```

Test invocation:
```bash
curl -X POST http://localhost:8080/api/sql \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT test_query_rhai() AS val;"}'
```

Expected output:
```json
[{"val": 100}]
```

---

## Step 2: Verify Python Backend Integration

Create the Python query function:
```sql
CREATE FUNCTION test_query_python() RETURNS INT
LANGUAGE PYTHON AS '
import oxibase
rows = oxibase.query("SELECT 200 as val;")
val = rows[0]["val"]
return val;
';
```

Test invocation:
```bash
curl -X POST http://localhost:8080/api/sql \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT test_query_python() AS val;"}'
```

Expected output:
```json
[{"val": 200}]
```

---

## Step 3: Verify PL/SQL Table Declaration and Loop Iteration

Create the PL/SQL loop function:
```sql
CREATE FUNCTION test_table_loop_plsql() RETURNS TEXT
LANGUAGE plsql AS '
DECLARE
    v_rows TABLE;
    v_row JSON;
    v_names TEXT := '''';
BEGIN
    v_rows := query_rows(''SELECT ''''Alice'''' AS name UNION SELECT ''''Bob'''' AS name'');
    FOR v_row IN v_rows LOOP
        v_names := v_names || v_row.name || '' '';
    END LOOP;
    RETURN v_names;
END;
';
```

Test invocation:
```bash
curl -X POST http://localhost:8080/api/sql \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT test_table_loop_plsql() AS val;"}'
```

Expected output:
```json
[{"val": "Alice Bob "}]
```
