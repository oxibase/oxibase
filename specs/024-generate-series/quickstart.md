# Quickstart: generate_series

The `generate_series` function allows you to generate a set of sequential values on the fly directly in the database.

## Usage

Generate numbers from 1 to 5:
```sql
SELECT * FROM generate_series(1, 5);
```

Generate numbers from 0 to 10 with a step of 2:
```sql
SELECT * FROM generate_series(0, 10, 2);
```

Generate backwards from 5 to 1:
```sql
SELECT * FROM generate_series(5, 1, -1);
```
