---
layout: default
title: CREATE VIEW
parent: DDL
grand_parent: SQL Commands
---

# CREATE VIEW

Creates a virtual table based on a SELECT statement.

#### Basic Syntax

```sql
CREATE VIEW view_name AS SELECT ...;
```

#### Examples

```sql
-- Simple view
CREATE VIEW active_products AS
SELECT * FROM products WHERE in_stock = true;

-- View with joins
CREATE VIEW order_details AS
SELECT o.id, c.name as customer, p.name as product, o.quantity
FROM orders o
JOIN customers c ON o.customer_id = c.id
JOIN products p ON o.product_id = p.id;

-- Query the view
SELECT * FROM active_products WHERE price > 100;
```
