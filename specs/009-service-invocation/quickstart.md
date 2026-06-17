# Quickstart: Exposing Procedures via HTTP

This feature allows you to invoke stored procedures directly over HTTP, turning your database into a fully-fledged backend.

## 1. Create a Stored Procedure

First, define a procedure in the database using SQL or any supported scripting language (like Rhai, JS, or Python).

```sql
CREATE PROCEDURE process_order(user_id INTEGER, amount INTEGER)
LANGUAGE RHAI AS $$
    let auth = oxibase::get_http_header("Authorization");
    if auth == () || auth == null {
        throw "Unauthorized";
    }
    
    // Process the order...
    return amount * 1.05; // Return amount with tax
$$;
```

## 2. Start the HTTP Server

Ensure the Oxibase server is running with the `server` feature enabled.

```bash
cargo run --features server -- serve --port 8080
```

## 3. Invoke via HTTP POST

Send a POST request to the `/api/rpc/process_order` endpoint with a JSON body matching the parameter names.

```bash
curl -X POST http://localhost:8080/api/rpc/process_order \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer my-secret-token" \
  -d '{"user_id": 42, "amount": 100}'
```

**Response:**

```json
{
  "result": 105.0
}
```
