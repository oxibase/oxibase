# Quickstart: Database-Driven Routes

This guide demonstrates how to serve a dynamic HTML page directly from Oxibase.

## 1. Start the Server

Start the Oxibase database with the server feature enabled:

```bash
cargo run --features server -- serve
```

## 2. Define a Template and Route

Using the Oxibase SQL CLI (or via HTTP API), create the system tables and insert your template and route:

```sql
-- Create system tables (if not already created automatically by the engine)
CREATE TABLE system_templates (name TEXT, content TEXT);
CREATE TABLE system_routes (method TEXT, path TEXT, template_name TEXT, context_query TEXT);

-- Create a table with some data
CREATE TABLE users (id INTEGER, username TEXT);
INSERT INTO users VALUES (1, 'alice'), (2, 'bob');

-- Insert a Jinja template
INSERT INTO system_templates VALUES (
  'user_list',
  '<h1>Users</h1><ul>{% for user in data %}<li>{{ user.username }}</li>{% endfor %}</ul>'
);

-- Map a route to the template and provide data via a query
INSERT INTO system_routes VALUES (
  'GET',
  '/users',
  'user_list',
  'SELECT username FROM users'
);
```

## 3. View the Page

Open your browser or use curl to visit the dynamically generated route:

```bash
curl http://localhost:8080/users
```

**Output**:
```html
<h1>Users</h1><ul><li>alice</li><li>bob</li></ul>
```
