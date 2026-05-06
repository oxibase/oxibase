# API Contract: Auto-API Layer

## Endpoints

### 1. Retrieve Data (GET)
**Endpoint:** `GET /api/:table`

**Query Parameters:**
- `select` (optional): Comma-separated list of columns to return (e.g., `id,name`). Defaults to `*`.
- `order` (optional): Comma-separated list of columns to sort by, with optional direction (e.g., `id.desc,name.asc`). Defaults to ascending if omitted.
- `limit` (optional): Integer, max number of rows to return.
- `offset` (optional): Integer, number of rows to skip.
- `[column]=eq.[value]` (optional): Exact match filtering.

**Success Response (200 OK):**
```json
[
  {
    "id": 1,
    "name": "Widget"
  }
]
```

**Error Responses:**
- `404 Not Found`: If `:table` does not exist.
- `500 Internal Server Error`: If the query execution fails.

### 2. Insert Data (POST)
**Endpoint:** `POST /api/:table`

**Request Body:**
A JSON object representing a single row to insert.
```json
{
  "id": 1,
  "name": "Widget"
}
```

**Success Response (201 Created):**
```json
{
  "rows_affected": 1
}
```

**Error Responses:**
- `404 Not Found`: If `:table` does not exist.
- `400 Bad Request`: If the JSON payload is malformed.
- `500 Internal Server Error`: If the insert execution fails.

### 3. Update Data (PATCH)
**Endpoint:** `PATCH /api/:table`

**Query Parameters:**
- `[column]=eq.[value]` (required): Exact match filtering to target the rows for update (e.g., `id=eq.1`).

**Request Body:**
A JSON object with the fields to update.
```json
{
  "name": "Super Widget"
}
```

**Success Response (200 OK):**
```json
{
  "rows_affected": 1
}
```

**Error Responses:**
- `404 Not Found`: If `:table` does not exist.
- `400 Bad Request`: If the JSON payload is malformed or filter is missing.
- `500 Internal Server Error`: If the update execution fails.

### 4. Delete Data (DELETE)
**Endpoint:** `DELETE /api/:table`

**Query Parameters:**
- `[column]=eq.[value]` (required): Exact match filtering to target the rows for deletion (e.g., `id=eq.1`).

**Success Response (200 OK):**
```json
{
  "rows_affected": 1
}
```

**Error Responses:**
- `404 Not Found`: If `:table` does not exist.
- `400 Bad Request`: If the filter is missing.
- `500 Internal Server Error`: If the delete execution fails.
