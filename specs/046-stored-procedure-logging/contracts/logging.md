# Interface & Grammar Contracts

## 1. Rhai Scripting API

The Rhai scripting engine exposes the following built-in function inside the `oxibase` namespace:

```typescript
oxibase::log(level: string, message: string);
```

- **Arguments**:
  - `level`: A string denoting log level. Valid cases: `"ERROR"`, `"WARN"`, `"INFO"`, `"DEBUG"`, `"TRACE"` (case-insensitive).
  - `message`: Any string representing the message.
- **Return Type**: `()` (unit / void)

---

## 2. Python Scripting API

The Python module `oxibase` exposes the following native function:

```python
import oxibase

oxibase.log(level: str, message: str) -> None
```

- **Arguments**:
  - `level`: Case-insensitive string representing the log level.
  - `message`: The message string.
- **Return Type**: `None`

---

## 3. PL/SQL Grammar Integration

The PL/SQL parser will accept the following statement structure:

```ebnf
LOG <level_identifier>, <expression>;
```

- **Level Identifier**: An unquoted identifier mapping to a valid level name (`INFO`, `WARN`, `ERROR`, `DEBUG`, `TRACE`).
- **Expression**: Any valid PL/SQL expression that can be evaluated to a text/string type.
- **Examples**:
  ```sql
  LOG INFO, 'Database started';
  LOG WARN, 'Attempt number: ' || attempt_count;
  ```
- **Syntax integration**:
  - Parsed within `Statement::parse` in `src/functions/plsql/parser.rs`.
  - Interpreted within `Statement::execute` in `src/functions/plsql/interpreter.rs`.
