# SQL API Contract

Base Path: `/api/sql`

## Execute Raw SQL Query

Executes arbitrary DDL or DML SQL statements.

- **URL**: `/api/sql`
- **Method**: `POST`
- **Content-Type**: `application/json`

### Request Body
```json
{
  "query": "SELECT * FROM public.users LIMIT 10;"
}
```

### Response (200 OK) - Result Set (e.g., SELECT)
```json
{
  "columns": ["id", "username", "email"],
  "rows": [
    {
      "id": 1,
      "username": "admin",
      "email": "admin@example.com"
    }
  ]
}
```

### Response (200 OK) - Execution Status (e.g., INSERT, UPDATE, CREATE)
```json
{
  "rows_affected": 1
}
```

### Response (400 Bad Request) - SQL Error
```json
{
  "error": {
    "code": "syntax_error",
    "message": "Syntax error near 'FROMM'"
  }
}
```