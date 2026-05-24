# Quickstart: Metadata API and Workspace App

## Installation

To deploy the Workspace App into your Oxibase instance, you must install the templates into a persistent database and then serve it. Execute the CLI command on demand:

```bash
# Run the installation command against a persistent database file
cargo run --features cli -- install-workspace -d "file:///tmp/oxibase.db"

# Start the Oxibase server using the same persistent database
cargo run --features server -- serve -d "file:///tmp/oxibase.db" --port 8080
```

## Accessing the Workspace

Once installed, open your web browser and navigate to:

```text
http://localhost:8080/workspace
```

## Using the Metadata API

You can now use the new structured endpoints for database management:

### List all tables
```bash
curl -X GET http://localhost:8080/api/meta/tables
```

### Create a new table
```bash
curl -X POST http://localhost:8080/api/meta/tables \
  -H "Content-Type: application/json" \
  -d '{
    "name": "employees",
    "columns": [
      {"name": "id", "type": "INTEGER", "is_nullable": false},
      {"name": "name", "type": "TEXT"}
    ]
  }'
```

### Execute Raw SQL
```bash
curl -X POST http://localhost:8080/api/sql \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * FROM employees"}'
```