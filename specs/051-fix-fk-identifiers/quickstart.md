# Schema-Qualified Foreign Keys Quickstart

This guide explains how to define and use foreign key constraints that span across schemas in Oxibase.

## 1. Prerequisites: Create Schemas

Before setting up cross-schema relationships, ensure the relevant schemas exist:

```sql
CREATE SCHEMA crm;
CREATE SCHEMA sales;
```

## 2. Define Table-level Cross-Schema Foreign Keys

Create a primary/parent table in one schema and a referencing/child table in a different schema:

```sql
-- Parent Table
CREATE TABLE crm.customers (
    id INTEGER PRIMARY KEY,
    name TEXT
);

-- Child Table
CREATE TABLE sales.orders (
    id INTEGER PRIMARY KEY,
    customer_id INTEGER,
    FOREIGN KEY (customer_id) REFERENCES crm.customers(id) ON DELETE CASCADE
);
```

## 3. Define Column-level Cross-Schema Foreign Keys

You can also specify the reference directly at the column level:

```sql
CREATE TABLE sales.invoices (
    id INTEGER PRIMARY KEY,
    customer_id INTEGER REFERENCES crm.customers(id) ON DELETE SET NULL
);
```

## 4. Unqualified References within Schema Contexts

If a child table is defined in a non-default schema and refers to an unqualified parent table, Oxibase automatically looks for the parent table in the same schema:

```sql
-- Parent and child in the same custom schema "sales"
CREATE TABLE sales.promotions (
    id INTEGER PRIMARY KEY,
    code TEXT UNIQUE
);

CREATE TABLE sales.orders (
    id INTEGER PRIMARY KEY,
    promo_code TEXT,
    -- Resolves dynamically to sales.promotions(code)
    FOREIGN KEY (promo_code) REFERENCES promotions(code)
);
```
