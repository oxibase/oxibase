# Quickstart: FROM-First Syntax

Oxibase now supports DuckDB-style "FROM-first" syntax, allowing you to write queries that start with the `FROM` clause. This aligns the query structure with the logical flow of data (from source to projection) and provides a faster way to explore data.

## Exploring Data (No SELECT)

If you omit the `SELECT` clause, Oxibase automatically assumes `SELECT *`. This is the fastest way to quickly inspect a table:

```sql
-- Standard SQL
SELECT * FROM users;

-- FROM-first shorthand
FROM users;
```

## Chaining Clauses

You can add any standard clauses (`WHERE`, `ORDER BY`, `LIMIT`) after the `FROM` clause in any order:

```sql
-- Filter and order users
FROM users WHERE age > 18 ORDER BY name LIMIT 10;
```

## Custom Projections (With SELECT)

When you need specific columns, add the `SELECT` clause after the `FROM` clause.

```sql
-- Standard SQL
SELECT id, email FROM users WHERE status = 'active';

-- FROM-first
FROM users WHERE status = 'active' SELECT id, email;
-- or
FROM users SELECT id, email WHERE status = 'active';
```

## Advanced Sources

The syntax works with any valid `FROM` expression, including joins, table functions, and `VALUES` clauses:

```sql
-- Using table functions
FROM generate_series(1, 5) AS s(num);

-- Using JOINs
FROM users u JOIN orders o ON u.id = o.user_id SELECT u.name, o.total;

-- Using VALUES
FROM (VALUES (1, 'Alice'), (2, 'Bob')) AS t(id, name);
```