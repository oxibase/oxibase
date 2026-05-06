# Feature Specification: Auto-API Layer

## 1. Feature Description

Implement an automatic REST API layer for Oxibase that dynamically exposes CRUD operations based on the underlying database schema. This feature leverages the `information_schema` to discover tables and endpoints on the fly, eliminating the need to manually define API routes for every table. The server will be built using Axum and run as part of the `oxibase serve` CLI command.

## Clarifications
### Session 2026-05-06
- Q: Which data mutation methods should we support in this Auto-API layer? → A: Full CRUD (GET, POST, PATCH, DELETE)
- Q: How complex should the horizontal filtering syntax be for this milestone? → A: Only the exact match `eq.` operator
- Q: Should we implement Vertical Filtering (`select`) and Ordering (`order`)? → A: Both `select` and `order`

## 2. User Scenarios & Testing

*   **Scenario 1: Starting the Server**
    *   *Action*: The user runs `oxibase serve --port 8080 --db file:///path/to/db`.
    *   *Expected Result*: The database boots up, binds to port 8080, and starts listening for HTTP requests without crashing.
*   **Scenario 2: Automatic Endpoint Discovery**
    *   *Action*: The user creates a new table `CREATE TABLE products (id INT, name TEXT)`. Then, the user sends a `GET /api/products` request.
    *   *Expected Result*: The server dynamically verifies `products` exists in the `information_schema` and returns a JSON array (initially empty) with a 200 OK status.
*   **Scenario 3: Rejecting Invalid Endpoints**
    *   *Action*: The user sends a request to `GET /api/non_existent_table`.
    *   *Expected Result*: The server checks the `information_schema`, finds no such table, and returns a 404 Not Found status.
*   **Scenario 4: Inserting Data via POST**
    *   *Action*: The user sends a POST request to `/api/products` with a JSON body `{"id": 1, "name": "Widget"}`.
    *   *Expected Result*: The server constructs and executes an `INSERT` statement. The data is persisted, and the server returns a success response (e.g., 201 Created) along with the number of affected rows.
*   **Scenario 5: Retrieving Data via GET with Pagination**
    *   *Action*: After inserting data, the user sends `GET /api/products?select=id,name&order=id.desc&limit=10&offset=0`.
    *   *Expected Result*: The server executes a `SELECT id, name FROM products ORDER BY id DESC LIMIT 10 OFFSET 0` query and returns the inserted row(s) mapped to native JSON objects.
*   **Scenario 6: Updating Data via PATCH**
    *   *Action*: The user sends `PATCH /api/products?id=eq.1` with a JSON body `{"name": "Super Widget"}`.
    *   *Expected Result*: The server executes an `UPDATE` statement and returns a success response with the affected row count.
*   **Scenario 7: Deleting Data via DELETE**
    *   *Action*: The user sends `DELETE /api/products?id=eq.1`.
    *   *Expected Result*: The server executes a `DELETE` statement and returns a success response.

## 3. Functional Requirements

*   **FR1**: The system must provide a new CLI subcommand `oxibase serve` that starts an HTTP server using Axum.
*   **FR2**: The HTTP server must accept `--port` (default 8080) and `--host` (default 127.0.0.1) arguments.
*   **FR3**: The server must expose a wildcard GET endpoint `GET /api/:table` that dynamically translates to a `SELECT * FROM :table` query.
*   **FR4**: The GET endpoint must support `limit` and `offset` query parameters for pagination, exact match filtering using the `eq.` operator (e.g., `?id=eq.1`), vertical filtering using the `select` parameter (e.g., `?select=id,name`), and ordering using the `order` parameter (e.g., `?order=id.desc`).
*   **FR5**: The server must expose a wildcard POST endpoint `POST /api/:table` that dynamically translates to an `INSERT INTO :table` query based on the JSON payload.
*   **FR6**: The server must expose a wildcard PATCH endpoint `PATCH /api/:table` that dynamically translates to an `UPDATE` query, updating fields provided in the JSON payload based on exact match filters (using `eq.`).
*   **FR7**: The server must expose a wildcard DELETE endpoint `DELETE /api/:table` that dynamically translates to a `DELETE` query based on exact match filters (using `eq.`).
*   **FR8**: The system must validate the existence of the `:table` parameter against the database's `information_schema.tables` before executing any query to prevent SQL injection and provide proper 404 responses.
*   **FR9**: Database `Value` types must be serialized to standard `serde_json::Value` objects so the API returns clean, standard JSON arrays.
*   **FR10**: The web server functionality must be gated behind a `server` Cargo feature.

## 4. Success Criteria

*   Users can launch the Oxibase server with a single CLI command.
*   Creating a table immediately exposes REST endpoints for it without restarting the server or adding code.
*   Attempting to access non-existent tables correctly returns a 404 error rather than a generic 500 error or SQL syntax error.
*   Data retrieved from the API is formatted as standard JSON arrays/objects, not internal string representations.

## 5. Key Entities (Optional)

*   `Database`: The core embedded database instance, wrapped in thread-safe references (`Arc`) to be shared across Axum request handlers.
*   `AppState`: The Axum application state holding the database connection and configuration.

## 6. Dependencies & Assumptions

*   **Dependencies**: Requires `axum`, `tokio`, `tower-http`, and `serde_json`.
*   **Assumptions**: We assume the REST API will initially map 1:1 with tables (no complex joins or custom views exposed by default in this initial implementation, although views that appear in `information_schema.tables` should work). We assume the POST payload structure directly matches the table column names. Authentication and authorization are out of scope for this specific Auto-API MVP layer (to be handled later via ABAC/RBAC).