# Architecture & Research Findings: Service Invocation via HTTP

## 1. HTTP Procedure Routing and Context

**Need:** How to implement Axum routing for `/api/rpc/:procedure_name` and pass JSON request body.
**Decision:** We will add a new `post` route to the `Router` in `src/server/mod.rs` mapping to a new handler `invoke_procedure` in `src/server/handlers.rs`.
**Rationale:** This matches the existing `Auto-API` pattern. The handler will extract `procedure_name` from the URL, parse the body as a JSON object, and use `FunctionRegistry` to look up the procedure and map JSON keys to ordered parameters.

## 2. Argument Mapping

**Need:** How to safely map a JSON payload to the stored procedure's arguments.
**Decision:** The handler will query the `FunctionRegistry` for the `StoredProcedure` using `get_procedure`. It will iterate through `procedure.parameters`, looking up each parameter's name in the JSON body. 
**Rationale:** We need to handle missing parameters (return 400 Bad Request) and data type conversions. Since the procedure parameters have defined types, the handler will attempt to convert `serde_json::Value` to `oxibase::Value` matching the required type.

## 3. Passing HTTP Headers via Built-in Function

**Need:** How to implement `get_http_header('Header-Name')`.
**Decision:** 
1. Create a thread-local storage `thread_local! { pub static HTTP_HEADERS: RefCell<Option<HashMap<String, String>>> = RefCell::new(None); }` in a shared location (e.g., `src/server/handlers.rs` or `src/functions/backends.rs`).
2. The `invoke_procedure` handler will collect request headers, place them in the thread-local storage using a closure `with_http_headers`, and then invoke `execute_call`.
3. Create a new scalar function `GetHttpHeaderFunction` in `src/functions/scalar/utility.rs` that reads from this thread-local storage.
**Rationale:** This avoids passing an explicit `ExecutionContext` payload through multiple layers of AST evaluation and cleanly decouples the database engine from the HTTP server while still enabling the feature.

## 4. Execution Bridge

**Need:** How to execute the procedure and return JSON.
**Decision:** The handler will construct a mock `CallStatement` AST node (or bypass AST and directly call `backend.execute_procedure`). Since `execute_call` in `query.rs` requires an `ExecutionContext` and handles transaction management properly, it's safer to either construct a `CallStatement` and pass it to `Executor::execute_statement`, or extract the logic of `execute_call` to be callable directly with a list of `Value` arguments.
**Alternative Considered:** Constructing a SQL string `CALL proc(?, ?)` and running `executor.execute_with_params()`. This is simpler and reuses the parser, avoiding manual AST construction. We will go with this approach: build the SQL string and use `execute_with_params`.