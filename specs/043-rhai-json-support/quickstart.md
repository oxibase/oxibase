# Quickstart: Rhai JSON Support

## Using JSON in Rhai Scripts

With the JSON support enabled, you can now seamlessly manipulate JSON data inside your Rhai triggers and functions.

### 1. Manual Parsing

If you have a JSON string (e.g. from a `TEXT` column) and want to parse it manually, you can use the built-in `parse_json` function:

```sql
CREATE FUNCTION extract_city(payload TEXT) RETURNS TEXT
LANGUAGE RHAI AS '
    let data = parse_json(payload);
    return data.address.city;
';

SELECT extract_city('{"address": {"city": "New York"}}');
-- Returns: "New York"
```

### 2. Transparent JSON Arguments

If your SQL function defines an argument as `JSON`, Oxibase will automatically parse it before passing it to your Rhai script as a dynamic object map. No manual parsing is required!

```sql
CREATE FUNCTION get_score(user_data JSON) RETURNS INTEGER
LANGUAGE RHAI AS '
    // user_data is natively a Rhai Map because the SQL type is JSON
    if user_data.is_active {
        return user_data.score;
    } else {
        return 0;
    }
';

SELECT get_score('{"is_active": true, "score": 95}'::JSON);
-- Returns: 95
```

### 3. Transparent JSON Returns

If your SQL function specifies `RETURNS JSON`, you can return a native Rhai Map or Array directly, and Oxibase will automatically serialize it back to a JSON value for SQL.

```sql
CREATE FUNCTION create_user_profile(name TEXT, age INTEGER) RETURNS JSON
LANGUAGE RHAI AS '
    // Create a native Rhai Map and return it directly
    return #{
        name: name,
        age: age,
        status: "new"
    };
';

SELECT create_user_profile('Alice', 28);
-- Returns SQL JSON: {"name": "Alice", "age": 28, "status": "new"}
```
