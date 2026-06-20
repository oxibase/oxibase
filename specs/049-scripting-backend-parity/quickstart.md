# Quickstart: Backend Parity

## Python Scripting

You can now pass JSON and TIMESTAMP data directly into Python functions, and they will be natively converted:

```python
CREATE FUNCTION process_event(event JSON, created_at TIMESTAMP) RETURNS JSON
LANGUAGE PYTHON AS '
    # event is a native Python dict
    # created_at is a native Python datetime object
    
    event["processed"] = True
    event["processing_time"] = created_at.isoformat()
    
    # Returning the dict converts it back to Oxibase JSON
    return event
'
```

## PL/SQL Additions

### JSON and TIMESTAMP Support

```plsql
DECLARE
    v_data JSON;
    v_time TIMESTAMP;
BEGIN
    v_data := CAST('{"key": "value"}' AS JSON);
    v_time := CAST('2026-06-20T10:00:00Z' AS TIMESTAMP);
END;
```

### Random Number Generation

```plsql
DECLARE
    v_rand FLOAT;
BEGIN
    v_rand := random();
    PRINT v_rand;
END;
```
