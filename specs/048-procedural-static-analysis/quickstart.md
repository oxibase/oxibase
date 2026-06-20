# Quickstart: AST-in-AST Script Analysis

This guide shows you how to use the `analyze_script` API to detect related database objects in stored procedures and scripts statically.

## 1. Rust API Usage

To analyze a stored procedure script from your Rust application, use `Database::analyze_script`:

```rust
use oxibase::api::Database;

let db = Database::open_in_memory()?;

// Analyze a Rhai stored procedure
let script = r#"
    oxibase::execute("SELECT * FROM schema_name.orders");
    oxibase::call("notify_customer()");
"#;

let objects = db.analyze_script(script, "rhai")?;

for obj in objects {
    println!("{}: {}", obj.object_type, obj.name);
}
// Outputs:
// Table: schema_name.orders
// Procedure: notify_customer
```

---

## 2. Supported Languages

### Rhai
```javascript
oxibase::execute("INSERT INTO audit_log (msg) VALUES ('started')");
oxibase::query("SELECT name FROM users WHERE id = 1");
```
- **Analyzed**: Table `audit_log`, Table `users`.

### Python
```python
import oxibase
oxibase.execute("UPDATE products SET stock = stock - 1 WHERE id = 1")
```
- **Analyzed**: Table `products`.

### PL/SQL
```sql
DECLARE
    v_id INTEGER;
BEGIN
    SELECT id INTO v_id FROM employees WHERE email = 'test';
END;
```
- **Analyzed**: Table `employees`.
