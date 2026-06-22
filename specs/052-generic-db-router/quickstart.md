# Quickstart: Generic Database-Driven Router

This quickstart guides you through registering and testing a dynamic, database-driven route in Oxibase Server.

## 1. Bootstrapping a Dynamic Route and Template

You can define routes and templates directly by executing standard SQL queries. Let's register a new route that queries a table of custom user notes by a dynamic path parameter.

### Step 1: Create a Custom Schema and Table
```sql
CREATE SCHEMA IF NOT EXISTS dev;
CREATE TABLE dev.notes (id INTEGER, title TEXT, content TEXT);
INSERT INTO dev.notes VALUES (1, 'First Note', 'Welcome to Oxibase! This is database-driven rendering.');
INSERT INTO dev.notes VALUES (2, 'Generic Routing', 'Notice how no Rust recompilation is needed!');
```

### Step 2: Register the Template
```sql
INSERT INTO interface.templates (name, content) 
VALUES (
    'note_view.html', 
    '<html>
       <body>
         <h1>Note Details (ID: {{ params.note_id }})</h1>
         {% for row in data %}
           <h3>{{ row.title }}</h3>
           <p>{{ row.content }}</p>
         {% else %}
           <p>No note found with that ID.</p>
         {% endfor %}
         <p><a href="/workspace">Back to Dashboard</a></p>
       </body>
     </html>'
);
```

### Step 3: Register the Dynamic Route with Named Parameters
```sql
INSERT INTO interface.routes (method, path, template_name, context_query)
VALUES (
    'GET', 
    '/dev/notes/{note_id}', 
    'note_view.html', 
    'SELECT title, content FROM dev.notes WHERE id = CAST(:note_id AS INTEGER)'
);
```

---

## 2. Accessing and Testing the Route

1. **Start the Oxibase Server**:
   ```bash
   cargo run --features "cli" -- bin oxibase server
   ```

2. **Send a request via curl**:
   ```bash
   curl http://localhost:8080/dev/notes/1
   ```

3. **Verify the Output**:
   The response will render the `note_view.html` template populated dynamically with data from row `1` of `dev.notes`:
   ```html
   <html>
     <body>
       <h1>Note Details (ID: 1)</h1>
       <h3>First Note</h3>
       <p>Welcome to Oxibase! This is database-driven rendering.</p>
       <p><a href="/workspace">Back to Dashboard</a></p>
     </body>
   </html>
   ```
