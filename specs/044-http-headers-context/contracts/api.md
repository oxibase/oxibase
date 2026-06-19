# Interface Contract: HTTP Headers Context API

This contract defines the interfaces exposed in both Python and PL/SQL scripting environments to access incoming HTTP request context.

## 1. Python VM Interface

### `oxibase` Module API

Python procedures and user-defined functions (UDFs) can access request headers by importing the built-in `oxibase` module.

#### Function Signature
```python
oxibase.get_http_header(header_name: str) -> str | None
```

#### Behavior
- **Case-Insensitivity**: The `header_name` argument lookup is case-insensitive.
- **HTTP Context Present**: Returns the string value of the header if found.
- **HTTP Context Absent / Missing Header**: Returns `None` if the header is not present or if the database is running outside an HTTP RPC context (e.g., standard SQL connection).

#### Example Usage
```python
import oxibase

auth_token = oxibase.get_http_header("Authorization")
if auth_token is not None:
    # Perform custom verification or business logic
    pass
```

---

## 2. PL/SQL Interpreter Interface

### Built-in Function API

PL/SQL procedures and scalar functions can call the built-in `get_http_header` function directly in expressions.

#### Function Signature
```sql
get_http_header(header_name TEXT) RETURNS TEXT
```

#### Behavior
- **Case-Insensitivity**: The `header_name` argument lookup is case-insensitive.
- **HTTP Context Present**: Returns the text value of the header if found.
- **HTTP Context Absent / Missing Header**: Returns `NULL` if the header is not present or if the database is running outside an HTTP RPC context.

#### Example Usage
```sql
DECLARE
    auth_token TEXT;
BEGIN
    auth_token := get_http_header('Authorization');
    IF auth_token IS NOT NULL THEN
        -- Perform logic
    END IF;
END;
```
