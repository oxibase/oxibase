# Quickstart: Auto-API Layer

## Running the Server

1.  Build the project with the `server` feature enabled (if it isn't enabled by default):
    ```bash
    cargo build --features server
    ```
2.  Start the Oxibase server, pointing it to a database file:
    ```bash
    ./target/debug/oxibase serve --db file:///tmp/mydb --port 8080
    ```

## Testing the Auto-API

1.  In another terminal, use the Oxibase CLI (REPL) to create a table and insert data:
    ```bash
    ./target/debug/oxibase repl --db file:///tmp/mydb
    > CREATE TABLE users (id INT, name TEXT);
    > INSERT INTO users (id, name) VALUES (1, 'Alice');
    ```
2.  Use `curl` to query the dynamically generated API endpoint:
    ```bash
    curl http://localhost:8080/api/users
    ```
    *Output:*
    ```json
    [{"id": 1, "name": "Alice"}]
    ```
3.  Use `curl` to insert new data via the API:
    ```bash
    curl -X POST http://localhost:8080/api/users \
         -H "Content-Type: application/json" \
         -d '{"id": 2, "name": "Bob"}'
    ```
4.  Verify the insertion:
    ```bash
    curl http://localhost:8080/api/users
    ```
