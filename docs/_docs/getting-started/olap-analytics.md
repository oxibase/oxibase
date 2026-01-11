---
title: OLAP Use Case 
layout: default
parent: Getting Started
nav_order: 3
---

# OLAP Analytics 

This guide demonstrates an Online Analytical Processing (OLAP) workflow using
Oxibase: creating a persistent database, defining a schema, loading data from a
file, performing analytical queries, and exporting results.

## Prerequisites

- Oxibase installed (see [Installation]({% link _docs/tutorials/installation.md %}))
- A CSV file with sample data (we'll create one)

## Step 1: Create a Persistent Database

Create a database file instead of using in-memory mode:

```bash
mkdir -p data
oxibase --db "file://./data/analytics.db"
```

This starts Oxibase with persistent storage in `./data/analytics.db`.

## Step 2: Create Schema

Define tables for sales data:

```sql
CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT,
    price DECIMAL(10,2)
);

CREATE TABLE customers (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE,
    region TEXT
);

CREATE TABLE sales (
    id INTEGER PRIMARY KEY,
    quantity INTEGER,
    unit_price DECIMAL(10,2),
    sale_date DATE,
    product_id INTEGER,
    customer_id INTEGER
);
```

## Step 3: Load Data from File


{: .warning-title }
> WIP
>
> Loading data from file is not yet supported. Please follow [this
> issue](https://github.com/oxibase/oxibase/issues/21) to get
> updates on the status of this feature.

Create sample data files and load them:

First, create `products.csv`:

```
id,name,category,price
1,Laptop,Electronics,999.99
2,Mouse,Electronics,29.99
3,Book,Media,19.99
4,Chair,Furniture,149.99
5,Headphones,Electronics,79.995,Headphones,Electronics,79.99
```

Create `customers.csv`:

```
id,name,email,region
1,Alice Johnson,alice@example.com,North
2,Bob Smith,bob@example.com,South
3,Carol Davis,carol@example.com,East
4,David Wilson,david@example.com,West
5,Eve Brown,eve@example.com,North
```

Create `sales.csv`:

```
id,product_id,customer_id,quantity,unit_price,sale_date
1,1,1,1,999.99,2024-01-15
2,2,1,2,29.99,2024-01-16
3,3,2,1,19.99,2024-01-20
4,4,3,1,149.99,2024-02-01
5,5,4,1,79.99,2024-02-05
6,1,5,1,999.99,2024-02-10
7,2,2,1,29.99,2024-02-15
8,3,3,3,19.99,2024-02-20
9,4,1,1,149.99,2024-03-01
10,5,5,2,79.99,2024-03-05
```

Load the data (assuming CSV files are in the current directory):

```sql
COPY products FROM 'products.csv' WITH CSV HEADER;
COPY customers FROM 'customers.csv' WITH CSV HEADER;
COPY sales FROM 'sales.csv' WITH CSV HEADER;
```

## Step 4: Analyze Data

Perform analytical queries:

### Sales by Category

```sql
SELECT
    p.category,
    COUNT(s.id) as total_sales,
    SUM(s.quantity * s.unit_price) as total_revenue,
    AVG(s.quantity * s.unit_price) as avg_sale_amount
FROM sales s
JOIN products p ON s.product_id = p.id
GROUP BY p.category
ORDER BY total_revenue DESC;
```

### Customer Analysis with Window Functions

```sql
SELECT
    c.name,
    c.region,
    COUNT(s.id) as purchase_count,
    SUM(s.quantity * s.unit_price) as total_spent,
    RANK() OVER (ORDER BY SUM(s.quantity * s.unit_price) DESC) as spending_rank,
    LAG(SUM(s.quantity * s.unit_price)) OVER (ORDER BY c.name) as prev_customer_total
FROM customers c
LEFT JOIN sales s ON c.id = s.customer_id
GROUP BY c.id, c.name, c.region
ORDER BY total_spent DESC;
```

### Time-based Analysis

```sql
SELECT
    strftime('%Y-%m', sale_date) as month,
    COUNT(*) as monthly_sales,
    SUM(quantity * unit_price) as monthly_revenue,
    SUM(SUM(quantity * unit_price)) OVER (ORDER BY strftime('%Y-%m', sale_date)) as cumulative_revenue
FROM sales
GROUP BY strftime('%Y-%m', sale_date)
ORDER BY month;
```

### Top Products by Revenue

```sql
WITH product_revenue AS (
    SELECT
        p.name,
        p.category,
        SUM(s.quantity * s.unit_price) as total_revenue,
        COUNT(s.id) as sales_count
    FROM products p
    JOIN sales s ON p.id = s.product_id
    GROUP BY p.id, p.name, p.category
)
SELECT
    name,
    category,
    total_revenue,
    sales_count,
    ROW_NUMBER() OVER (ORDER BY total_revenue DESC) as revenue_rank
FROM product_revenue
ORDER BY total_revenue DESC;
```

## Step 5: Export Results

Export analytical results to CSV:

{: .warning-title }
> WIP
>
> Export data to a file is not yet supported. Please follow [this
> issue](https://github.com/oxibase/oxibase/issues/21) to get
> updates on the status of this feature.

```sql
-- Export sales summary by category
COPY (
    SELECT
        p.category,
        COUNT(s.id) as total_sales,
        SUM(s.quantity * s.unit_price) as total_revenue
    FROM sales s
    JOIN products p ON s.product_id = p.id
    GROUP BY p.category
    ORDER BY total_revenue DESC;
) TO 'category_sales.csv';

-- Export customer analysis
COPY (
    SELECT
        c.name,
        COUNT(s.id) as purchase_count,
        SUM(s.quantity * s.unit_price) as total_spent
    FROM customers c
    LEFT JOIN sales s ON c.id = s.customer_id
    GROUP BY c.id, c.name
    ORDER BY total_spent DESC;
) TO 'customer_analysis.csv' 
  WITH (
      FORMAT CSV, 
      HEADER, 
      DELIMITER ',', 
      ENCODING UTF8
);
-- Export top products by revenue
COPY (
    WITH product_revenue AS (
        SELECT
            p.name,
            p.category,
            SUM(s.quantity * s.unit_price) as total_revenue,
            COUNT(s.id) as sales_count
        FROM products p
        JOIN sales s ON p.id = s.product_id
        GROUP BY p.id, p.name, p.category
    )
    SELECT
        name,
        category,
        total_revenue,
        sales_count,
        ROW_NUMBER() OVER (ORDER BY total_revenue DESC) as revenue_rank
    FROM product_revenue
    ORDER BY total_revenue DESC;
) TO 'top_products.parquet' WITH (FORMAT PARQUET, COMPRESSION SNAPPY);
```
