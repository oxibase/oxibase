# Quickstart: Multiline Code Blocks

## Creating a Function with Dollar Quotes

The standard way to provide multiline scripts for database functions without worrying about escaping internal single quotes:

```sql
CREATE FUNCTION process_data(data JSON) RETURNS JSON
LANGUAGE js
AS $$
    // Look! No need to escape single quotes like 'this'
    const result = JSON.parse(data);
    result.status = 'processed';
    return JSON.stringify(result);
$$;
```

## Using Tagged Dollar Quotes

If your script happens to contain `$$`, you can use tagged dollar quotes to avoid premature termination:

```sql
CREATE FUNCTION nested_example() RETURNS TEXT
LANGUAGE python
AS $py$
    def test():
        return "Using $$ inside Python is now safe!"
    
    return test()
$py$;
```

## Using Triple Backticks

For a modern markdown-like experience, you can enclose the code in triple backticks:

```sql
CREATE FUNCTION format_log(msg TEXT) RETURNS TEXT
LANGUAGE js
AS ```
    return `[LOG]: ${msg}`;
```;
```
