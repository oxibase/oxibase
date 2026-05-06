-- setup.sql

-- First, ensure the schema and tables exist just in case we are running this before the server starts
CREATE SCHEMA routes;
CREATE SCHEMA templates;
CREATE TABLE routes.definitions (method TEXT, path TEXT, template_name TEXT, context_query TEXT);
CREATE TABLE templates.source (name TEXT, content TEXT);

-- Insert templates
INSERT INTO templates.source (name, content) VALUES ('layout.html', '
<!DOCTYPE html>
<html>
<head>
    <title>Oxibase CMS</title>
    <style>
        body { font-family: sans-serif; margin: 40px; }
        h1 { color: #333; }
        ul { padding-left: 20px; }
        .user-card { border: 1px solid #ccc; padding: 10px; margin-bottom: 10px; border-radius: 4px; }
    </style>
</head>
<body>
    <header>
        <h1>Oxibase Dynamic Pages</h1>
        <nav><a href="/">Home</a> | <a href="/users">Users</a></nav>
    </header>
    <hr>
    <main>
        {% block content %}{% endblock %}
    </main>
</body>
</html>
');

INSERT INTO templates.source (name, content) VALUES ('index.html', '
{% extends "layout.html" %}
{% block content %}
    <h2>Welcome to the Home Page!</h2>
    <p>This page is served entirely from the <code>templates.source</code> table in the database.</p>
    <p>Try visiting the <a href="/users">Users</a> page to see dynamic data injection.</p>
{% endblock %}
');

INSERT INTO templates.source (name, content) VALUES ('users.html', '
{% extends "layout.html" %}
{% block content %}
    <h2>Users List</h2>
    <p>This page fetches dynamic data from the <code>users</code> table.</p>
    
    {% if data %}
        {% for user in data %}
        <div class="user-card">
            <strong>ID:</strong> {{ user.id }} <br>
            <strong>Name:</strong> {{ user.name }} <br>
            <strong>Role:</strong> {{ user.role }}
        </div>
        {% endfor %}
    {% else %}
        <p>No users found in the database.</p>
    {% endif %}
{% endblock %}
');

-- Create a dummy users table
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, role TEXT);
INSERT INTO users (id, name, role) VALUES (1, 'Alice', 'Admin');
INSERT INTO users (id, name, role) VALUES (2, 'Bob', 'Editor');
INSERT INTO users (id, name, role) VALUES (3, 'Charlie', 'Viewer');

-- Map routes
INSERT INTO routes.definitions (method, path, template_name, context_query) VALUES ('GET', '/', 'index.html', NULL);
INSERT INTO routes.definitions (method, path, template_name, context_query) VALUES ('GET', '/users', 'users.html', 'SELECT id, name, role FROM users ORDER BY id');
