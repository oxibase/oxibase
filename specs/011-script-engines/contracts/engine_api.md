# Interface Contract: Python & JS Trigger API

Triggers executing under `LANGUAGE python` and `LANGUAGE js` expose the following native interfaces to the developer.

## Global Variables

| Variable | Description | Availability | Read/Write |
| :--- | :--- | :--- | :--- |
| `NEW` | Represents the post-mutation row state. | `INSERT`, `UPDATE` | Read + Write |
| `OLD` | Represents the pre-mutation row state. | `UPDATE`, `DELETE` | Read-only |

### Property Access Behavior
*   **Success**: `NEW.column_name` returns the value natively cast to the scripting engine's primitive type (e.g., Integer to Python `int` or JS `Number`).
*   **Mutation**: `NEW.column_name = "value"` coerces the input to the internal DB schema type. If coercion fails, a runtime exception is thrown.
*   **Failure**: Accessing an invalid column name throws `AttributeError` (Python) or returns `undefined` (JS, per language specs).

## `oxibase` Module API

To perform side-effects (like writing to audit logs), scripts can use the `oxibase` module.

### Python
```python
import oxibase
oxibase.execute("INSERT INTO audit_log (id) VALUES (1)")
```

### JavaScript
```javascript
// oxibase is injected globally
oxibase.execute("INSERT INTO audit_log (id) VALUES (1)");
```
