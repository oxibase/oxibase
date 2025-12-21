---
layout: default
title: PostgreSQL Server
parent: Getting Started
nav_order: 4
---

# PostgreSQL Server

OxiBase includes a built-in PostgreSQL-compatible server that allows you to connect to your databases using standard PostgreSQL client tools like `psql`, pgAdmin, or any PostgreSQL driver.

## Overview

The OxiBase server implements the PostgreSQL wire protocol (pgwire), enabling seamless connectivity with existing PostgreSQL tools and applications. This server runs as a separate binary and provides network access to OxiBase databases.

## Building the Server

The server is available as an optional feature. To build it:

```bash
# Build with PostgreSQL server support
cargo build --release --features pg-server

# Or install with server support
cargo install oxibase --features pg-server
```

## Starting the Server

Run the server binary with your desired configuration:

```bash
# Basic usage with in-memory database
cargo run --features pg-server --bin server -- --db memory://

# With a persistent database
cargo run --features pg-server --bin server -- --db "file:///path/to/data"

# Custom host and port
cargo run --features pg-server --bin server -- --host 127.0.0.1 --port 5433 --db "file:///data/mydb"
```

### Command Line Options

| Option | Default | Description |
|--------|---------|-------------|
| `--host` | 127.0.0.1 | IP address to bind the server to |
| `--port` | 5433 | Port number to listen on |
| `--db` | memory:// | Database connection string (see [Connection Strings](connection-strings)) |

## Connecting with psql

Once the server is running, connect using `psql`:

```bash
# Connect to the default configuration
psql -h localhost -p 5433 -d postgres

# Connect to a specific database file
psql -h localhost -p 5433 -d postgres -U postgres
```

Since the server currently uses trust authentication, no password is required.

## Example Usage

### Basic Query

```bash
# Start the server
cargo run --features pg-server --bin server -- --db memory://

# In another terminal, connect and query
psql -h localhost -p 5433 -d postgres
```

```sql
-- Create a table
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    created_at TIMESTAMP
);

-- Insert some data
INSERT INTO users (id, name, email, created_at) VALUES
(1, 'Alice', 'alice@example.com', NOW()),
(2, 'Bob', 'bob@example.com', NOW()),
(3, 'Charlie', 'charlie@example.com', NOW());

-- Query the data
SELECT * FROM users;

-- Filter and sort
SELECT name, email FROM users WHERE id > 1 ORDER BY name;
```

### Using with Applications

The server is compatible with any PostgreSQL driver. For example, with Python:

```python
import psycopg2

# Connect to OxiBase server
conn = psycopg2.connect(
    host="localhost",
    port=5433,
    database="postgres",
    user="postgres"
)

cursor = conn.cursor()
cursor.execute("SELECT * FROM users")
results = cursor.fetchall()

for row in results:
    print(row)

conn.close()
```

## Data Types

OxiBase supports the following PostgreSQL-compatible data types:

| OxiBase Type | PostgreSQL Type | Example |
|--------------|-----------------|---------|
| INTEGER | INTEGER/BIGINT | 42 |
| FLOAT | REAL/DOUBLE PRECISION | 3.14159 |
| TEXT | TEXT/VARCHAR | 'Hello World' |
| BOOLEAN | BOOLEAN | TRUE |
| TIMESTAMP | TIMESTAMPTZ | '2023-12-01 12:00:00+00' |
| JSON | JSONB | '{"key": "value"}' |
| NULL | NULL | NULL |

## Current Limitations

The server implementation is in active development. Current limitations include:

- **DML Response Format**: INSERT, UPDATE, DELETE operations return informational messages instead of standard PostgreSQL command completion tags
- **Authentication**: Only trust authentication is supported (no password required)
- **Prepared Statements**: Not yet implemented
- **Transactions**: Basic transaction support, but some advanced features may be limited
- **Performance**: All query results are collected in memory before streaming

## Security Considerations

- The server currently uses trust authentication - any connection is accepted
- Data is transmitted in plain text (no SSL/TLS encryption)
- No connection limits or rate limiting implemented
- Suitable for development and testing environments

## Troubleshooting

### Connection Refused

Ensure the server is running and accessible:

```bash
# Check if the server is listening
netstat -tlnp | grep 5433

# Or use lsof
lsof -i :5433
```

### Authentication Failed

The server currently accepts any username/password combination. If you see authentication errors, verify:

- You're using the correct host and port
- The server is running with `--features pg-server`
- No firewall is blocking the connection

### Query Errors

OxiBase supports a subset of PostgreSQL SQL syntax. Common issues:

- Some PostgreSQL-specific features may not be implemented
- Data type handling might differ from PostgreSQL
- Check the [SQL Features](../sql-features/) documentation for supported syntax

## Next Steps

- [Quick Start Tutorial](quickstart) - Learn basic OxiBase operations
- [Connection Strings](connection-strings) - Database connection options
- [SQL Commands](../sql-commands/) - Comprehensive SQL reference
- [Architecture](../architecture/) - Learn how OxiBase works internally

For more advanced usage, see the [API Reference](api-reference) and [SQL Features](../sql-features/) documentation.