# Quickstart: HTTP Headers Context for Python and PL/SQL

This feature enables stored procedures and functions written in Python and PL/SQL to read HTTP request headers when executed via HTTP endpoints.

## 1. Python Quickstart

Define a procedure in Python that reads the `Authorization` header:

```sql
CREATE PROCEDURE get_token_py(OUT res TEXT)
LANGUAGE python AS '
import oxibase
res = oxibase.get_http_header("Authorization")
';
```

Invoke the procedure over HTTP:

```bash
curl -X POST http://localhost:8080/api/rpc/get_token_py \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer python-token-abc" \
  -d '{}'
```

**Response:**
```json
{
  "res": "Bearer python-token-abc"
}
```

---

## 2. PL/SQL Quickstart

Define a procedure in PL/SQL that reads the `Authorization` header:

```sql
CREATE PROCEDURE get_token_plsql(OUT res TEXT)
LANGUAGE plsql AS '
BEGIN
    res := get_http_header(''Authorization'');
END;
';
```

Invoke the procedure over HTTP:

```bash
curl -X POST http://localhost:8080/api/rpc/get_token_plsql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer plsql-token-xyz" \
  -d '{}'
```

**Response:**
```json
{
  "res": "Bearer plsql-token-xyz"
}
```
