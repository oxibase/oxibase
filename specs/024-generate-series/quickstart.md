# Quickstart: generate_series

The `generate_series` function allows you to generate a set of sequential values on the fly directly in the database. It supports integers, floats, dates, and timestamps.

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

Generate a series of dates:
```sql
SELECT * FROM generate_series('2024-01-01', '2024-01-05', '1 day');
```

Use as a scalar function to return an array:
```sql
SELECT generate_series(1, 5);
```