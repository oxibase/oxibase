---
layout: default
title: SQL Functions
parent: How to Guides
nav_order: 1
---

# How to use SQL Functions
{: .no_toc}

---

#### Table of Contents
{: .no_toc}

1. TOC
{:toc}

---


This document provides an overview of the SQL functions supported by Oxibase. For detailed documentation on each function type, see the dedicated guides below.

## Function Types

### Aggregate Functions
Aggregate functions perform calculations on sets of values to return a single result, typically used with GROUP BY clauses for data summarization.

**[View Aggregate Functions Guide →]({% link _docs/references/functions/aggregate-functions.md %})**

### Scalar Functions
Scalar functions operate on individual values and return a single result. They include string manipulation, numeric operations, date/time processing, type conversion, and more.

**[View Scalar Functions Guide →]({% link _docs/references/functions/scalar-functions.md %})**

### Window Functions
Window functions perform calculations across related rows without grouping them into a single output row, enabling advanced analytical queries.

**[View Window Functions Guide →]({% link _docs/references/functions/window-functions.md %})**

## Usage Examples

### Basic Function Usage
```sql
-- Scalar function
SELECT UPPER(name), LENGTH(description) FROM products;

-- Aggregate function
SELECT category, COUNT(*), AVG(price) FROM products GROUP BY category;

-- Window function
SELECT name, salary, ROW_NUMBER() OVER (ORDER BY salary DESC) as rank FROM employees;
```

### Function Chaining
Functions can be nested for complex operations:
```sql
SELECT ROUND(AVG(price), 2) FROM products;
```

### Functions in Queries
Functions can be used in SELECT, WHERE, GROUP BY, HAVING, and ORDER BY clauses:
```sql
SELECT DATE_TRUNC('month', order_date) as month, SUM(total) as sales
FROM orders
WHERE order_date >= DATE_SUB(NOW(), 1, 'year')
GROUP BY DATE_TRUNC('month', order_date)
HAVING SUM(total) > 10000
ORDER BY month;
```

## Additional Resources

- **[User-Defined Functions](user-defined-functions.md)**: Creating custom functions with scripting backends
- **[Stored Procedures](procedures.md)**: Multi-statement procedures (planned feature)
- **[SQL Features](../sql-features/)**: Advanced SQL capabilities including ROLLUP/CUBE operations

## Implementation Notes

OxiBase's function system is designed for performance and extensibility:
- **Modular Architecture**: Functions are organized by type for easy maintenance
- **Type Safety**: Arguments are validated at parse time
- **Multiple Backends**: Support for different scripting languages for UDFs
- **Optimization**: Many functions can be pushed down to storage layer

For detailed implementation information, see the individual function guides.
