---
layout: default
title: Using the Auto-API JSON Layer
parent: How to Guides 
nav_order: 4
---

# Using the Auto-API JSON Layer

Oxibase features a zero-configuration "Auto-API" layer. When you run Oxibase with the embedded HTTP server enabled, every table and view in your database is automatically exposed as a RESTful JSON endpoint.

This allows frontend applications or external services to perform full CRUD (Create, Read, Update, Delete) operations directly against the database via HTTP without writing any backend routing logic.

## Starting the Server

Start Oxibase with the `serve` command:

```bash
oxibase serve -d file:///path/to/my_db --port 8080
```

## Endpoint Structure

The Auto-API layer exposes your tables under the `/api/` prefix.

`[HTTP METHOD] /api/:table_name`
`POST /api/rpc/:procedure_name`

### 1. Read Data (GET)

To fetch data from a table, make a `GET` request to the table's endpoint.

```bash
curl http://127.0.0.1:8080/api/users
```

**Response:**
```json
[
  { "id": 1, "name": "Alice", "role": "Admin" },
  { "id": 2, "name": "Bob", "role": "Editor" }
]
```

#### Query Parameters

The `GET` endpoint supports several powerful query parameters to shape your data:

- **`select`**: Choose specific columns to return (comma-separated).
  - Example: `/api/users?select=id,name`
- **`limit` & `offset`**: Paginate your results.
  - Example: `/api/users?limit=10&offset=20`
- **`order`**: Sort the results. Suffix the column with `.asc` or `.desc`.
  - Example: `/api/users?order=id.desc,name.asc`
- **Filtering (`col=eq.val`)**: Filter rows by exact match using the `eq.` (equals) operator.
  - Example: `/api/users?role=eq.Admin`

**Combined Example:**
```bash
# Get the names of the top 5 admins, ordered alphabetically
curl "http://127.0.0.1:8080/api/users?role=eq.Admin&select=name&order=name.asc&limit=5"
```

### 2. Insert Data (POST)

To insert new rows into a table, send a `POST` request with a JSON object payload representing the row.

```bash
curl -X POST http://127.0.0.1:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"id": 3, "name": "Charlie", "role": "Viewer"}'
```

**Response:**
```json
{ "rows_affected": 1 }
```

*Note: Oxibase automatically maps standard JSON types (strings, numbers, booleans, nulls) to their corresponding internal SQL data types.*

### 3. Update Data (PATCH)

To modify existing rows, send a `PATCH` request. You **must** include a filter in the query string (e.g., `?id=eq.X`) to target specific rows. The JSON payload should contain only the columns you wish to update.

```bash
curl -X PATCH "http://127.0.0.1:8080/api/users?id=eq.3" \
  -H "Content-Type: application/json" \
  -d '{"role": "Editor"}'
```

**Response:**
```json
{ "rows_affected": 1 }
```

### 4. Delete Data (DELETE)

To remove rows from a table, send a `DELETE` request. Like the `PATCH` method, you **must** include a filter in the query string to prevent accidental full-table deletions.

```bash
curl -X DELETE "http://127.0.0.1:8080/api/users?id=eq.3"
```

**Response:**
```json
{ "rows_affected": 1 }
```

### 5. Service Invocation / Stored Procedures (POST /api/rpc)

To invoke a stored procedure over HTTP, you can use the `/api/rpc/:procedure_name` endpoint. This allows you to expose complex business logic and multi-statement transactions to the web without writing a custom backend.

Send a `POST` request with a JSON object payload. The keys in the JSON object must match the input parameter names of the stored procedure.

```bash
# Assuming a procedure: CREATE PROCEDURE update_inventory(product_id INT, quantity INT, OUT success BOOLEAN)
curl -X POST http://127.0.0.1:8080/api/rpc/update_inventory \
  -H "Content-Type: application/json" \
  -d '{"product_id": 123, "quantity": 10}'
```

**Response:**
The response is a JSON object containing the values of the `OUT` or `INOUT` parameters returned by the procedure.
```json
{
  "success": true
}
```

#### Accessing HTTP Metadata from Procedures

When a procedure is invoked via `/api/rpc/`, it can read the incoming HTTP request headers using the built-in `oxibase::get_http_header('header_name')` SQL function. This is useful for passing authentication tokens, user agents, or custom metadata securely into your business logic.

```sql
CREATE PROCEDURE get_my_ip(OUT ip_address TEXT)
LANGUAGE plsql AS $$
BEGIN
    -- Read a header injected by a reverse proxy
    ip_address = get_http_header('x-forwarded-for');
END;
$$;
```

## Security & Architecture Notes

- **Zero Copy Engine**: The JSON API layer reads rows directly from the MVCC storage engine and streams the serialized JSON to the client, ensuring high throughput and minimal memory overhead.
- **Transactions**: Every API request (`POST`, `PATCH`, `DELETE`) is implicitly wrapped in an ACID-compliant transaction. If an error occurs during the operation, it is completely rolled back.
- **Errors**: Invalid queries, type mismatches, or missing tables will return appropriate HTTP status codes (e.g., `404 Not Found` for missing tables, `400 Bad Request` for missing filters on updates/deletes, `500 Internal Server Error` for SQL execution errors) along with a JSON body describing the error `{"error": "..."}`.
