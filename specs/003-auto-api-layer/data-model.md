# Data Model: Auto-API Layer

## Entities

The Auto-API layer does not introduce new persistent database entities. Instead, it interacts with existing entities:

1.  **`information_schema.tables`**: Used for validating table existence.
2.  **User Tables**: Any table created by the user (e.g., `products`, `users`). The API dynamically generates routes for these.

## State Transitions

- The HTTP server acts as a stateless gateway to the database.
- Each HTTP request (GET/POST) translates into a discrete SQL transaction (handled implicitly or explicitly by the `Database` API).

## Validation Rules

1.  **Route Validation**: The requested `/:table` must exist in `information_schema.tables`. If not, return HTTP 404.
2.  **Payload Validation**: For POST requests, the JSON payload must be parsed and translated into an `INSERT` statement. If the payload format is invalid or types don't match the table schema, the underlying database execution will fail, and the API should return an appropriate HTTP 400 Bad Request error.
