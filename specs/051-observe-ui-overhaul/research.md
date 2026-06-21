# Research: Observability UI Overhaul

## 1. Safe Dynamic SQL Filtering

To avoid SQL injection vulnerabilities when executing filters across levels, log messages, and trace IDs, we must dynamically construct query expressions using exclusively parameterized positional bindings (`?`).

### Log Queries
When a search term, trace ID, or severity level is provided:
```rust
let mut conditions = Vec::new();
let mut values = Vec::new();

if let Some(level) = level {
    conditions.push("level = ?".to_string());
    values.push(Value::text(level));
}
if let Some(search) = search {
    conditions.push("(message LIKE ? OR target LIKE ?)".to_string());
    values.push(Value::text(format!("%{}%", search)));
    values.push(Value::text(format!("%{}%", search)));
}
```

---

## 2. Unpoly Infinite Scroll Implementation

Unpoly handles SPA-like page updates and supports infinite scrolling using appending markers.
Inside `workspace_observe_logs.html`, we specify a container and a trigger element:
```html
<div id="log-list">
    <!-- Log rows -->
    <div class="log-row">...</div>
    
    <!-- Infinite scroll trigger -->
    <a href="/workspace/observe/logs?offset=50" 
       up-target="#log-list" 
       up-scroll="reveal" 
       up-transition="none"
       class="loading-trigger">Load more...</a>
</div>
```
Unpoly reads the link, fetches the subsequent set, and replaces/appends the matched nodes dynamically without a full-page repaint.

---

## 3. Gantt Tree Assembler Algorithm (JavaScript)

To avoid heavy server-side processing, we retrieve a flat list of all trace spans ordered by `start_time` and construct a hierarchical tree directly inside the user's browser:

```javascript
function buildSpanTree(spans) {
    const spanMap = {};
    const roots = [];

    // Initialize map
    spans.forEach(span => {
        span.children = [];
        spanMap[span.span_id] = span;
    });

    // Link parents & children
    spans.forEach(span => {
        const parent = spanMap[span.parent_span_id];
        if (parent) {
            parent.children.push(span);
        } else {
            roots.push(span);
        }
    });

    return roots;
}
```

This ensures we handle orphaned spans gracefully (they become top-level roots) and keeps rendering highly performant.

---

## 4. Auto-Refresh Polling (`up-poll`)

Unpoly has native polling support. Simply adding an `up-poll` attribute to any container handles repeating AJAX updates behind the scenes:
```html
<div id="observe-container" up-poll up-interval="5000">
   <!-- Auto-reloaded contents every 5 seconds -->
</div>
```
This requires zero custom intervals or socket connections, fitting perfectly within the monolithic design.
