---
layout: default
title: SELECT
parent: DQL
grand_parent: SQL Commands
---

# SELECT

<div id="rrdiagram"></div>
<script>
  document.addEventListener("DOMContentLoaded", function() {
    var diagram = Diagram([
      Sequence([
        Keyword("SELECT"),
        Optional(Choice(0, [Keyword("DISTINCT"), Keyword("ALL")])),
        OneOrMore(NonTerminal("select_list"), Keyword(",")),
        Optional(Sequence([Keyword("FROM"), OneOrMore(NonTerminal("table_reference"), Keyword(","))])),
        Optional(Sequence([Keyword("WHERE"), NonTerminal("condition")])),
        Optional(Sequence([Keyword("GROUP BY"), OneOrMore(NonTerminal("expression"), Keyword(","))])),
        Optional(Sequence([Keyword("HAVING"), NonTerminal("condition")])),
        Optional(Sequence([Keyword("ORDER BY"), OneOrMore(NonTerminal("expression"), Keyword(","))])),
        Optional(Sequence([Keyword("LIMIT"), NonTerminal("count"), Optional(Sequence([Keyword("OFFSET"), NonTerminal("offset")]))]))
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  });
</script>

The SELECT statement retrieves data from one or more tables.

#### Basic Syntax

```sql
SELECT [DISTINCT] column1, column2, ...
FROM table_name
[WHERE condition]
[GROUP BY column1, ... | ROLLUP(column1, ...) | CUBE(column1, ...)]
[HAVING condition]
[ORDER BY column1 [ASC|DESC] [NULLS FIRST|NULLS LAST], ...]
[LIMIT count [OFFSET offset]]
```

#### Parameters

- **DISTINCT**: Removes duplicate rows from the result
- **column1, column2, ...**: Columns to retrieve; use `*` for all columns
- **table_name**: The table to query
- **WHERE condition**: Filter condition
- **GROUP BY**: Groups rows by specified columns
- **ROLLUP/CUBE**: Multi-dimensional aggregation (see [ROLLUP and CUBE]({% link _docs/references/sql-features/rollup-cube.md %}))
- **HAVING**: Filter applied to groups
- **ORDER BY**: Sorting of results (`NULLS FIRST` or `NULLS LAST` to control NULL placement)
- **LIMIT**: Maximum rows to return
- **OFFSET**: Number of rows to skip

#### Examples

```sql
-- Basic query
SELECT id, name, price FROM products;

-- Filtering
SELECT * FROM products WHERE price > 50.00 AND category = 'Electronics';

-- Sorting
SELECT * FROM products ORDER BY price DESC, name ASC;

-- Pagination
SELECT * FROM customers LIMIT 10 OFFSET 20;

-- Unique values
SELECT DISTINCT category FROM products;

-- Aggregation
SELECT category, AVG(price) AS avg_price, COUNT(*) as count
FROM products
GROUP BY category;

-- Filtering groups
SELECT category, COUNT(*) AS product_count
FROM products
GROUP BY category
HAVING COUNT(*) > 5;
```

#### JOIN Operations

Oxibase supports all standard JOIN types:

```sql
-- INNER JOIN
SELECT p.name, c.name AS category
FROM products p
INNER JOIN categories c ON p.category_id = c.id;

-- LEFT JOIN
SELECT c.name, o.id AS order_id
FROM customers c
LEFT JOIN orders o ON c.id = o.customer_id;

-- RIGHT JOIN
SELECT c.name, o.id AS order_id
FROM customers c
RIGHT JOIN orders o ON c.id = o.customer_id;

-- FULL OUTER JOIN
SELECT c.name, o.id
FROM customers c
FULL OUTER JOIN orders o ON c.id = o.customer_id;

-- CROSS JOIN
SELECT p.name, c.name
FROM products p
CROSS JOIN colors c;
```

See [JOIN Operations]({% link _docs/references/sql-features/join-operations.md %}) for detailed documentation.

#### Subqueries

Using subqueries in various clauses (both correlated and non-correlated):

```sql
-- Scalar subquery
SELECT name, price,
       (SELECT AVG(price) FROM products) as avg_price
FROM products;

-- IN subquery
SELECT * FROM customers
WHERE id IN (SELECT DISTINCT customer_id FROM orders);

-- EXISTS subquery (correlated)
SELECT * FROM customers c
WHERE EXISTS (SELECT 1 FROM orders o WHERE o.customer_id = c.id);

-- NOT IN subquery
SELECT * FROM products
WHERE id NOT IN (SELECT product_id FROM discontinued_items);

-- Correlated subquery in WHERE
SELECT * FROM employees e1
WHERE salary > (SELECT AVG(salary) FROM employees e2 WHERE e2.department = e1.department);

-- ANY/ALL subquery
SELECT * FROM products WHERE price > ALL (SELECT price FROM products WHERE category = 'Books');

-- Derived table (subquery in FROM)
SELECT * FROM (SELECT id, name FROM products WHERE price > 100) AS expensive;
```

See [Subqueries]({% link _docs/references/sql-features/subqueries.md %}) for detailed documentation.

#### Common Table Expressions (CTEs)

```sql
-- Simple CTE
WITH high_value_orders AS (
    SELECT * FROM orders WHERE total > 1000
)
SELECT * FROM high_value_orders;

-- Multiple CTEs
WITH
customer_totals AS (
    SELECT customer_id, SUM(total) as total_spent
    FROM orders
    GROUP BY customer_id
),
vip_customers AS (
    SELECT * FROM customer_totals WHERE total_spent > 10000
)
SELECT c.name, ct.total_spent
FROM customers c
JOIN vip_customers ct ON c.id = ct.customer_id;

-- Recursive CTE
WITH RECURSIVE numbers AS (
    SELECT 1 as n
    UNION ALL
    SELECT n + 1 FROM numbers WHERE n < 10
)
SELECT * FROM numbers;
```

See [Common Table Expressions]({% link _docs/references/sql-features/common-table-expressions.md %}) for detailed documentation.

#### Set Operations

```sql
-- UNION (removes duplicates)
SELECT name FROM customers
UNION
SELECT name FROM suppliers;

-- UNION ALL (keeps duplicates)
SELECT name FROM customers
UNION ALL
SELECT name FROM suppliers;

-- INTERSECT
SELECT id FROM table1
INTERSECT
SELECT id FROM table2;

-- EXCEPT
SELECT id FROM table1
EXCEPT
SELECT id FROM table2;
```

#### Temporal Queries (AS OF)

Query historical data at a specific point in time:

```sql
-- Query data as of a specific timestamp
SELECT * FROM orders AS OF TIMESTAMP '2024-01-15 10:30:00';

-- Query data as of current time
SELECT * FROM inventory AS OF TIMESTAMP NOW();
```

See [Temporal Queries]({% link _docs/references/sql-features/temporal-queries.md %}) for detailed documentation.
