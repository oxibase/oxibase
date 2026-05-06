---
layout: default
title: Creating Dynamic Web Pages with Jinja
parent: How to Guides 
nav_order: 3
---

# Creating Dynamic Web Pages with Jinja

Oxibase features a powerful embedded HTTP server that can not only expose JSON APIs but also serve dynamic HTML web pages directly from the database using the [Minijinja](https://docs.rs/minijinja/latest/minijinja/) template engine.

This guide explains how to define routes, execute queries for context data, and store Jinja templates directly in your database using SQL, allowing you to build CMS-like applications with zero downtime deployments.

## Prerequisites

To serve web pages, you must run Oxibase with the HTTP server enabled using the `serve` command:

```bash
oxibase serve -d file:///path/to/my_db
```

This starts the server on `127.0.0.1:8080` by default.

## The System Schemas

When the server starts, it automatically provisions two internal system tables to manage web pages:

1. **`templates.source`**: Stores raw Jinja template HTML.
   - `name` (TEXT): The unique filename/identifier for the template.
   - `content` (TEXT): The HTML content containing Jinja syntax.
2. **`routes.definitions`**: Maps HTTP paths to your templates and optionally executes SQL to provide data.
   - `method` (TEXT): The HTTP method (`GET`, `POST`, etc.).
   - `path` (TEXT): The URL path to intercept (e.g., `/`, `/dashboard`).
   - `template_name` (TEXT): Foreign key referencing a template in `templates.source`.
   - `context_query` (TEXT): Optional SQL query. The results are injected into the template context.

## Step 1: Storing a Simple Template

To create a new web page, insert your HTML code into the `templates.source` table using standard SQL.

```sql
INSERT INTO templates.source (name, content) 
VALUES ('about.html', '
<!DOCTYPE html>
<html>
<head>
    <title>About Us</title>
</head>
<body>
    <h1>About Oxibase</h1>
    <p>This page was rendered directly from the database!</p>
</body>
</html>
');
```

## Step 2: Mapping a Route

Once the template exists in the database, you can expose it to the web by mapping it to a URL path in the `routes.definitions` table:

```sql
INSERT INTO routes.definitions (method, path, template_name, context_query) 
VALUES ('GET', '/about', 'about.html', NULL);
```

You can now visit `http://127.0.0.1:8080/about` in your browser to see the rendered HTML page!

## Step 3: Injecting Dynamic Data

The true power of database-driven routing comes from the `context_query` column. If you provide a SQL query here, Oxibase will execute it automatically when a user visits the path, and inject the results into the Jinja template under the `data` variable.

First, let's create a table with some application data:

```sql
CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price FLOAT);
INSERT INTO products VALUES (1, 'Laptop', 1200.00);
INSERT INTO products VALUES (2, 'Keyboard', 85.50);
INSERT INTO products VALUES (3, 'Mouse', 45.00);
```

Next, write a template that iterates over the `data` array:

```sql
{% raw %}
INSERT INTO templates.source (name, content) 
VALUES ('products.html', '
<h1>Our Products</h1>
<ul>
  {% for product in data %}
    <li><strong>{{ product.name }}</strong>: ${{ product.price }}</li>
  {% else %}
    <li>No products available.</li>
  {% endfor %}
</ul>
');
{% endraw %}
```

Finally, bind the route and provide the SQL query:

```sql
INSERT INTO routes.definitions (method, path, template_name, context_query) 
VALUES (
    'GET', 
    '/products', 
    'products.html', 
    'SELECT name, price FROM products ORDER BY price DESC'
);
```

When a user accesses `/products`, Oxibase executes the `SELECT` query in real-time, serializes the resulting rows to JSON, and renders the `products.html` Jinja template.

## Template Composition (Inheritance)

Oxibase's embedded Jinja engine natively understands relationships between templates stored in the database. You can use standard Jinja tags like `{% raw %}{% extends %}{% endraw %}` and `{% raw %}{% block %}{% endraw %}` to create reusable layouts.

**1. Create a Base Layout:**
```sql
{% raw %}
INSERT INTO templates.source (name, content) VALUES ('layout.html', '
<html>
<head><title>My App</title></head>
<body>
    <nav><a href="/">Home</a></nav>
    <main>
        {% block content %}{% endblock %}
    </main>
</body>
</html>
');
{% endraw %}
```

**2. Create a Child Template:**
```sql
{% raw %}
INSERT INTO templates.source (name, content) VALUES ('home.html', '
{% extends "layout.html" %}
{% block content %}
    <h2>Welcome to the Home Page!</h2>
{% endblock %}
');
{% endraw %}
```

When you route a path to `home.html`, the engine will automatically query the database for `layout.html` and compile the full page on the fly.

## Live Updates

Because both routing definitions and HTML templates are read transactionally from the database using Oxibase's MVCC engine, you can edit your live application dynamically using `UPDATE` statements. No server restart is required.

```sql
-- Fix a typo in your live website
UPDATE templates.source 
SET content = REPLACE(content, 'Wlecome', 'Welcome') 
WHERE name = 'home.html';
```

If you delete a route from the database, the server instantly begins returning a standard `404 Not Found` HTTP status for that path:

```sql
DELETE FROM routes.definitions WHERE path = '/old-page';
```
