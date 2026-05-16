# Quickstart: PL/SQL Scalar Functions

This guide demonstrates how to define and use native PL/SQL scalar functions in Oxibase.

## Defining a Function

You can define a scalar function using `LANGUAGE plsql` or `LANGUAGE sql`. The function can execute control flow and must end with a `RETURN` statement if it returns a value.

```sql
CREATE FUNCTION calculate_discount(price FLOAT, discount_pct FLOAT) RETURNS FLOAT
LANGUAGE plsql
AS $$
BEGIN
    IF discount_pct < 0 OR discount_pct > 100 THEN
        RETURN 0.0;
    END IF;
    
    RETURN price * (discount_pct / 100.0);
END;
$$;
```

## Using the Function

Invoke the function inside a standard `SELECT` query:

```sql
SELECT calculate_discount(150.0, 20.0) AS discount;
```

## Expected Behavior
- The function will parse the PL/SQL block.
- Upon execution, it evaluates arguments and logic.
- When `RETURN <expr>` is hit, it immediately terminates the function and yields the expression's value to the caller.
