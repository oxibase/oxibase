---
title: "OLTP & OLAP Pizza Simulation"
layout: default
parent: Tutorials
nav_order: 5
---

# OLTP & OLAP Pizza Simulation: Decoupled Architectural Patterns

This tutorial demonstrates how to design a modern, decoupled database system inside Oxibase by building an end-to-end, automated transactional and analytical pipeline. 

By separating our workloads into **`pizza_tx`** (Online Transaction Processing / OLTP) and **`pizza_analytics`** (Online Analytical Processing / OLAP), we can maintain high-performance transactional updates while computing rich business intelligence, all inside the same embedded engine.

We will leverage the following core features of Oxibase:
* **Relational Schemas & Foreign Keys**: Restricting and maintaining data integrity across tables using schema-qualified constraints.
* **User-Defined Functions (UDFs)**: Writing custom, side-effect-free scalar functions using Rhai to standardize business logic.
* **Event-Driven Triggers**: Validating incoming transactions (`BEFORE INSERT`) and executing real-time data replication (`AFTER INSERT`) across schemas.
* **Stored Procedures (PL/SQL & Rhai)**: Simulating transactions and executing cross-schema rollups.
* **Background Task Automation (CRON Schedules)**: Automating the transaction workload and periodic aggregations in the background.
* **Advanced Analytical Queries**: Creating reporting views utilizing **Common Table Expressions (CTEs)**, **Window Functions** (`RANK() OVER`, `SUM() OVER`), and multi-dimensional aggregates (`GROUP BY ROLLUP`).

---

## 1. The Transactional Schema (`pizza_tx`)

The transactional schema houses tables responsible for day-to-day business operations (customers, orders, menu items, and pizza customizations).

```sql
CREATE SCHEMA IF NOT EXISTS pizza_tx;
```

### Relational Table Design with Schema-Qualified Foreign Keys

We enforce strict referential integrity between tables using schema-qualified references:

```sql
CREATE TABLE pizza_tx.customer (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT,
    address TEXT,
    phone TEXT
);

CREATE TABLE pizza_tx.credit_card (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT,
    number TEXT,
    expiration TEXT,
    UNIQUE (name, number)
);

CREATE TABLE pizza_tx.customer_credit_card (
    customer_id INTEGER,
    credit_card_id INTEGER,
    FOREIGN KEY (customer_id) REFERENCES pizza_tx.customer(id) ON DELETE CASCADE,
    FOREIGN KEY (credit_card_id) REFERENCES pizza_tx.credit_card(id) ON DELETE CASCADE
);

CREATE TABLE pizza_tx.customer_order (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    customer_id INTEGER,
    credit_card_id INTEGER,
    total_price FLOAT,
    FOREIGN KEY (customer_id) REFERENCES pizza_tx.customer(id) ON DELETE CASCADE,
    FOREIGN KEY (credit_card_id) REFERENCES pizza_tx.credit_card(id) ON DELETE SET NULL
);
```

To support pizza customizations (sizes, toppings, and drinks), we use join tables and set up strict lookup constraints:

```sql
CREATE TABLE pizza_tx.drinks (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT,
    price FLOAT
);

CREATE TABLE pizza_tx.sizes (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT,
    price FLOAT
);

CREATE TABLE pizza_tx.toppings (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT
);

CREATE TABLE pizza_tx.pizzas (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    size_id INTEGER,
    topping_1_id INTEGER,
    topping_2_id INTEGER,
    FOREIGN KEY (size_id) REFERENCES pizza_tx.sizes(id) ON DELETE RESTRICT,
    FOREIGN KEY (topping_1_id) REFERENCES pizza_tx.toppings(id) ON DELETE RESTRICT,
    FOREIGN KEY (topping_2_id) REFERENCES pizza_tx.toppings(id) ON DELETE RESTRICT
);

CREATE TABLE pizza_tx.customer_pizzas (
    customer_order_id INTEGER,
    pizza_id INTEGER,
    FOREIGN KEY (customer_order_id) REFERENCES pizza_tx.customer_order(id) ON DELETE CASCADE,
    FOREIGN KEY (pizza_id) REFERENCES pizza_tx.pizzas(id) ON DELETE CASCADE
);

CREATE TABLE pizza_tx.customer_drinks (
    customer_order_id INTEGER,
    drink_id INTEGER,
    FOREIGN KEY (customer_order_id) REFERENCES pizza_tx.customer_order(id) ON DELETE CASCADE,
    FOREIGN KEY (drink_id) REFERENCES pizza_tx.drinks(id) ON DELETE CASCADE
);
```

---

## 2. Operational Integrity & Real-Time Sync via Triggers

We protect the operational database from dirty data and bridge the gap to the analytical warehouse in real-time using event-driven triggers.

### Data Validation (`BEFORE INSERT`)

A validation trigger in **Rhai** ensures that newly created customer records have complete details and use a standard phone exchange:

```sql
CREATE TRIGGER validate_customer_phone
    BEFORE INSERT ON pizza_tx.customer
    FOR EACH ROW
    LANGUAGE rhai
AS '
    if oxibase.ctx.new.phone == "" {
        throw "Phone number is mandatory";
    }
    let phone_str = oxibase.ctx.new.phone;
    if !phone_str.contains("555-") {
        throw "Invalid phone format. Must be a standard 555- exchange.";
    }
';
```

### Real-time Event Replication (`AFTER INSERT`)

An audit trigger replicates freshly inserted transactional orders instantly into our analytical events log:

```sql
CREATE TRIGGER replicate_order_event
    AFTER INSERT ON pizza_tx.customer_order
    FOR EACH ROW
    LANGUAGE rhai
AS '
    let order_id = oxibase.ctx.new.id;
    let cust_id = oxibase.ctx.new.customer_id;
    let price = oxibase.ctx.new.total_price;
    let query = "INSERT INTO pizza_analytics.order_events_log (order_id, customer_id, total_price) VALUES (" + to_string(order_id) + ", " + to_string(cust_id) + ", " + to_string(price) + ")";
    oxibase::execute(query);
';
```

---

## 3. The Analytical Schema (`pizza_analytics`)

The analytical schema decouples analytical views and rollup caches from the primary OLTP layer, ensuring that massive BI queries do not slow down live transactions.

```sql
CREATE SCHEMA IF NOT EXISTS pizza_analytics;
```

We create the destination schemas for event replication and periodically consolidated summaries:

```sql
CREATE TABLE pizza_analytics.order_events_log (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    order_id INTEGER,
    customer_id INTEGER,
    total_price FLOAT,
    replicated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE pizza_analytics.daily_sales_summary (
    summary_date TEXT PRIMARY KEY,
    total_orders INTEGER,
    total_revenue FLOAT
);
```

---

## 4. Business Logic via User-Defined Functions (UDFs)

We declare dynamic scalar functions inside the `pizza_analytics` schema using the **Rhai** engine. These functions are side-effect-free and can be plugged directly into standard queries.

### Function 1: Customer Revenue Categorization
```sql
CREATE FUNCTION pizza_analytics.categorize_revenue(total FLOAT) RETURNS TEXT 
LANGUAGE rhai AS '
    if total < 15.0 {
        return "BRONZE";
    } else if total < 30.0 {
        return "SILVER";
    } else {
        return "GOLD";
    }
';
```

### Function 2: Delivery Logic Evaluator
```sql
CREATE FUNCTION pizza_analytics.free_delivery_eligible(total FLOAT) RETURNS BOOLEAN 
LANGUAGE rhai AS '
    return total >= 20.0;
';
```

---

## 5. Stored Procedures: Simulation & Warehousing

Stored procedures in Oxibase can have output parameters, handle transactional side-effects, and run multiple DML operations in isolated environments.

### Procedural Warehousing Rollup in PL/SQL

We write our warehouse rollup synchronization logic using native **PL/SQL** (PostgreSQL procedural dialect). It queries the `pizza_tx` tables, aggregates performance indicators, and upserts them into `pizza_analytics.daily_sales_summary`:

```sql
CREATE PROCEDURE pizza_analytics.sync_daily_summary()
LANGUAGE plsql
AS $$
BEGIN
    DELETE FROM pizza_analytics.daily_sales_summary WHERE summary_date = 'TODAY';
    
    INSERT INTO pizza_analytics.daily_sales_summary (summary_date, total_orders, total_revenue)
    SELECT 
        'TODAY', 
        COUNT(*), 
        COALESCE(SUM(total_price), 0.0) 
    FROM pizza_tx.customer_order;
END;
$$;
```

### Transaction Simulator in Rhai

A second procedure simulates real pizza store activity, generating randomized order flows and calculating total prices procedurally:

```sql
CREATE PROCEDURE pizza_tx.simulate_random_order() 
LANGUAGE rhai 
AS '
    oxibase::execute("INSERT INTO pizza_tx.customer_order (customer_id, credit_card_id, total_price) SELECT CAST(FLOOR(RANDOM() * 3) + 1 AS INTEGER), 1, 0.0");

    oxibase::execute("INSERT INTO pizza_tx.pizzas (size_id, topping_1_id, topping_2_id) SELECT CAST(FLOOR(RANDOM() * 4) + 1 AS INTEGER), CAST(FLOOR(RANDOM() * 12) + 1 AS INTEGER), CAST(FLOOR(RANDOM() * 12) + 1 AS INTEGER)");
    oxibase::execute("INSERT INTO pizza_tx.customer_pizzas (customer_order_id, pizza_id) SELECT (SELECT MAX(id) FROM pizza_tx.customer_order), (SELECT MAX(id) FROM pizza_tx.pizzas)");

    oxibase::execute("INSERT INTO pizza_tx.customer_drinks (customer_order_id, drink_id) SELECT (SELECT MAX(id) FROM pizza_tx.customer_order), CAST(FLOOR(RANDOM() * 5) + 1 AS INTEGER)");

    oxibase::execute("UPDATE pizza_tx.customer_order SET total_price = COALESCE((SELECT SUM(s.price) FROM pizza_tx.customer_pizzas cp JOIN pizza_tx.pizzas p ON cp.pizza_id = p.id JOIN pizza_tx.sizes s ON p.size_id = s.id WHERE cp.customer_order_id = (SELECT MAX(id) FROM pizza_tx.customer_order)), 0.0) + COALESCE((SELECT SUM(d.price) FROM pizza_tx.customer_drinks cd JOIN pizza_tx.drinks d ON cd.drink_id = d.id WHERE cd.customer_order_id = (SELECT MAX(id) FROM pizza_tx.customer_order)), 0.0) WHERE id = (SELECT MAX(id) FROM pizza_tx.customer_order)");

    oxibase::log("INFO", "Simulated transactional pizza order successfully created.");
';
```

---

## 6. Automating Execution via CRON Task Schedules

We set up background execution schedules using Oxibase's native CRON-based scheduler to run our pipeline automatically:

```sql
-- Simulate transactional pizza orders every minute
CREATE SCHEDULE simulate_pizza_orders 
CRON '0 * * * * * *' 
AS 'CALL pizza_tx.simulate_random_order()';

-- Sync and compile daily sales aggregates every minute at the 30s mark
CREATE SCHEDULE sync_analytics_daily 
CRON '30 * * * * * *' 
AS 'CALL pizza_analytics.sync_daily_summary()';
```

---

## 7. Reporting & Business Intelligence (OLAP Views)

Finally, we expose consolidated reporting layers to our downstream data-consumers and analytics tools.

### View 1: Clean Order Manifest with Custom Functions
This view uses our custom UDFs to present orders in a structured format:

```sql
CREATE VIEW pizza_analytics.v_order_details AS
SELECT 
    co.id AS order_id,
    c.name AS customer_name,
    co.total_price,
    pizza_analytics.free_delivery_eligible(co.total_price) AS free_delivery,
    pizza_analytics.categorize_revenue(co.total_price) AS customer_tier
FROM pizza_tx.customer_order co
JOIN pizza_tx.customer c ON co.customer_id = c.id;
```

To query the view:
```sql
SELECT * FROM pizza_analytics.v_order_details;
```

### View 2: Customer Lifetime Value (CLV) via CTEs and Window Functions
We compute individual customer value rankings and running cumulative sales totals dynamically:

```sql
CREATE VIEW pizza_analytics.v_customer_lifetime_value AS
WITH customer_totals AS (
    SELECT 
        customer_id,
        COUNT(id) AS order_count,
        SUM(total_price) AS total_spend
    FROM pizza_tx.customer_order
    GROUP BY customer_id
)
SELECT 
    c.id,
    c.name,
    ct.order_count,
    ct.total_spend,
    RANK() OVER (ORDER BY ct.total_spend DESC) AS revenue_rank,
    SUM(ct.total_spend) OVER (ORDER BY ct.total_spend DESC) AS cumulative_sales
FROM pizza_tx.customer c
LEFT JOIN customer_totals ct ON c.id = ct.customer_id;
```

To query the view:
```sql
SELECT * FROM pizza_analytics.v_customer_lifetime_value;
```

### View 3: Size & Topping Preferences using Multi-dimensional ROLLUP
We calculate a complete hierarchical breakdown of topping revenues by size including subtotals and grand totals in a single query:

```sql
CREATE VIEW pizza_analytics.v_revenue_by_size_and_topping AS
SELECT 
    s.name AS size_name,
    t.name AS topping_name,
    COUNT(co.id) AS total_orders,
    SUM(co.total_price) AS total_revenue
FROM pizza_tx.customer_order co
JOIN pizza_tx.customer_pizzas cp ON co.id = cp.customer_order_id
JOIN pizza_tx.pizzas p ON cp.pizza_id = p.id
JOIN pizza_tx.sizes s ON p.size_id = s.id
JOIN pizza_tx.toppings t ON p.topping_1_id = t.id
GROUP BY ROLLUP(s.name, t.name);
```

To query the view:
```sql
SELECT * FROM pizza_analytics.v_revenue_by_size_and_topping;
```
