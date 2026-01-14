---
layout: default
title: Subqueries
parent: Tutorials
nav_order: 1
---

# Using Subqueries

This guide provides a quick tutorial on using subqueries in Oxibase SQL
statements.

## What are Subqueries?

Subqueries are SQL queries nested within another query. They allow you to use
the result of one query as input for another, enabling more complex data
operations.

## Basic IN Subquery Example

Here's a simple example that demonstrates the power of subqueries:

```sql
-- Create sample tables
CREATE TABLE customers (
    id INTEGER PRIMARY KEY,
    name TEXT,
    country TEXT
);

CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    customer_id INTEGER,
    total FLOAT
);

-- Insert sample data
INSERT INTO customers VALUES 
    (1, 'Alice', 'USA'),
    (2, 'Bob', 'Canada'),
    (3, 'Charlie', 'USA');

INSERT INTO orders VALUES 
    (1, 1, 100.0),
    (2, 2, 200.0),
    (3, 1, 150.0);

-- Find all orders from US customers using a subquery
-- Returns orders 1 and 3 (Alice's orders)
SELECT * FROM orders 
WHERE customer_id IN (
    SELECT id FROM customers WHERE country = 'USA'
);
```

## EXISTS/NOT EXISTS Example

Check if related records exist without retrieving them:

```sql
-- Find customers who have placed at least one order
SELECT name FROM customers c
WHERE EXISTS (
    SELECT 1 FROM orders o WHERE o.customer_id = c.id
);

-- Find customers who have never ordered
SELECT name FROM customers c
WHERE NOT EXISTS (
    SELECT 1 FROM orders o WHERE o.customer_id = c.id
);
```

## Scalar Subqueries Example

Use subqueries that return a single value in comparisons:

```sql
-- Create logs table
CREATE TABLE logs (
    id INTEGER PRIMARY KEY,
    message TEXT,
    timestamp TEXT,
    priority TEXT
);

-- Insert sample data
INSERT INTO logs VALUES
    (1, 'System error', '2023-01-01 10:00:00', 'high'),
    (2, 'User action', '2023-01-15 14:30:00', 'low'),
    (3, 'Warning', '2023-02-01 09:15:00', 'medium');

-- Find orders above average
SELECT * FROM orders
WHERE total > (SELECT AVG(total) FROM orders);

-- Delete old records below a threshold
DELETE FROM logs
WHERE timestamp < (SELECT MIN(timestamp) FROM logs WHERE priority = 'high');
```

## Common Use Cases

### 1. Filtering Based on Another Table

```sql
-- Create sample tables
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT,
    last_login TEXT
);

CREATE TABLE user_sessions (
    id INTEGER PRIMARY KEY,
    user_id INTEGER,
    session_data TEXT
);

-- Insert sample data
INSERT INTO users VALUES
    (1, 'Alice', '2023-10-01'),
    (2, 'Bob', '2023-12-01'),
    (3, 'Charlie', '2023-09-15');

INSERT INTO user_sessions VALUES
    (1, 1, 'data1'),
    (2, 3, 'data2');

-- Delete inactive user data
DELETE FROM user_sessions
WHERE user_id IN (
    SELECT id FROM users WHERE last_login < DATE('now', '-90 days')
);
```

### 2. Bulk Updates Based on Conditions

```sql
-- Create sample tables
CREATE TABLE categories (
    id INTEGER PRIMARY KEY,
    name TEXT,
    tier TEXT
);

CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT,
    category_id INTEGER,
    discount FLOAT
);

-- Insert sample data
INSERT INTO categories VALUES
    (1, 'Electronics', 'premium'),
    (2, 'Books', 'standard');

INSERT INTO products VALUES
    (1, 'Laptop', 1, 0.0),
    (2, 'Novel', 2, 0.0),
    (3, 'Phone', 1, 0.0);

-- Apply discount to premium category products
UPDATE products
SET discount = 0.20
WHERE category_id IN (
    SELECT id FROM categories WHERE tier = 'premium'
);
```

### 3. Finding Missing Records

```sql
-- Find customers without any orders
SELECT * FROM customers 
WHERE id NOT IN (
    SELECT DISTINCT customer_id FROM orders
);
```

### 4. Existence Checks

```sql
-- Alter products table
ALTER TABLE products ADD COLUMN in_stock BOOLEAN DEFAULT true;

-- Create sample table
CREATE TABLE inventory (
    id INTEGER PRIMARY KEY,
    product_id INTEGER,
    quantity INTEGER
);

-- Insert sample data
INSERT INTO inventory VALUES
    (1, 1, 10),
    (2, 2, 0),
    (3, 3, 5);

-- Update product availability
UPDATE products
SET in_stock = false
WHERE NOT EXISTS (
    SELECT 1 FROM inventory WHERE product_id = products.id AND quantity > 0
);
```

### 5. Correlated Subqueries

```sql
-- Create sample table
CREATE TABLE employees (
    id INTEGER PRIMARY KEY,
    name TEXT,
    department TEXT,
    salary FLOAT
);

-- Insert sample data
INSERT INTO employees VALUES
    (1, 'Alice', 'Engineering', 80000.0),
    (2, 'Bob', 'Engineering', 75000.0),
    (3, 'Charlie', 'Sales', 70000.0),
    (4, 'David', 'Sales', 65000.0);

-- Find employees earning above their department average
SELECT name, department, salary
FROM employees e1
WHERE salary > (
    SELECT AVG(salary) FROM employees e2 WHERE e2.department = e1.department
);

-- Get each customer's total order amount
SELECT
    c.name,
    (SELECT SUM(total) FROM orders o WHERE o.customer_id = c.id) as total_spent
FROM customers c;
```

### 6. Derived Tables (Subqueries in FROM)

```sql
-- Join with aggregated data
SELECT c.name, stats.order_count, stats.total_spent
FROM customers c
JOIN (
    SELECT customer_id, COUNT(*) as order_count, SUM(total) as total_spent
    FROM orders
    GROUP BY customer_id
) AS stats ON c.id = stats.customer_id;
```

## Best Practices

1. **Keep subqueries simple**: Complex subqueries can impact performance
2. **Use indexes**: Ensure columns used in subquery WHERE clauses are indexed
3. **Consider alternatives**: Sometimes a JOIN might be more efficient than a subquery

## Supported Features

Oxibase fully supports:
- ✅ IN and NOT IN subqueries
- ✅ EXISTS and NOT EXISTS operators
- ✅ Scalar subqueries (returning single values)
- ✅ Correlated subqueries (referencing outer query)
- ✅ Derived tables (subqueries in FROM clause)
- ✅ Subqueries in SELECT, UPDATE, and DELETE statements
- ✅ Subqueries in UPDATE SET clause
- ✅ ANY/SOME and ALL operators

## Next Steps

- Read the full [Subqueries documentation]({% link _docs/sql-features/subqueries.md %})
- Learn about [JOIN operations]({% link _docs/sql-features/join-operations.md %}) as an alternative
- Explore [SQL Commands reference]({% link _docs/references/sql-commands.md %})
