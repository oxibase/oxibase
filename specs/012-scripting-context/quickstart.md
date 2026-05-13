# Quickstart: Using the New Trigger Context API

Triggers in Oxibase now expose their row data under the `oxibase.ctx` object instead of using global `OLD` and `NEW` variables.

## Python

```python
CREATE TRIGGER prevent_negative
    BEFORE UPDATE ON accounts
    FOR EACH ROW
    LANGUAGE python
AS '
import oxibase

if oxibase.ctx.new["balance"] < 0:
    # Mutating the proxy saves to the DB automatically
    oxibase.ctx.new["balance"] = 0
';
```

## JavaScript (Boa)

```javascript
CREATE TRIGGER uppercase_name
    BEFORE INSERT ON users
    FOR EACH ROW
    LANGUAGE js
AS '
    // Mutate the string directly
    oxibase.ctx.new.name = oxibase.ctx.new.name.toUpperCase();
';
```

## Rhai

```rust
CREATE TRIGGER ensure_audit
    AFTER UPDATE ON inventory
    FOR EACH ROW
    LANGUAGE rhai
AS '
    if oxibase.ctx.old.quantity != oxibase.ctx.new.quantity {
        oxibase::execute("INSERT INTO log ...");
    }
';
```