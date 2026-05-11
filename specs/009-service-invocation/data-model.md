# Data Model & Interface Contracts: Service Invocation

## Interface Contracts

### HTTP API: Invoke Procedure

**Endpoint**: `POST /api/rpc/:procedure_name`

**Request Headers**:
- `Content-Type: application/json`
- Any custom headers will be available to the procedure via `get_http_header()`.

**Request Body**:
A JSON object where keys match the stored procedure's parameter names.
```json
{
  "param1": "value",
  "param2": 123
}
```

**Response (Success - 200 OK)**:
```json
{
  "result": <Value>
}
```
If the procedure has OUT parameters, they may be returned as an object.

**Response (Error - 400 Bad Request)**:
```json
{
  "error": "Missing parameter 'param1'"
}
```

**Response (Error - 404 Not Found)**:
```json
{
  "error": "Procedure 'my_proc' not found"
}
```

**Response (Error - 500 Internal Server Error)**:
```json
{
  "error": "Execution failed: division by zero"
}
```

## Internal State Additions

### Thread-Local HTTP Context

To support `get_http_header()`, a thread-local variable will be introduced (likely in `src/functions/context.rs` or similar):

```rust
thread_local! {
    pub static HTTP_HEADERS: RefCell<Option<HashMap<String, String>>> = RefCell::new(None);
}
```

### New Scalar Function

**`GetHttpHeaderFunction`**:
- **Name**: `get_http_header`
- **Arguments**: 1 (String: Header name)
- **Returns**: String (Header value) or NULL if not found / not in HTTP context.