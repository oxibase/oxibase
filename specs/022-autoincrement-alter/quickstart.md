# Using ALTER TABLE to add Constraints

Oxibase allows you to evolve your schema by adding constraints to existing columns using the `ALTER TABLE ... MODIFY COLUMN` statement.

## Adding AUTOINCREMENT

You can add an `AUTOINCREMENT` constraint to an existing `INTEGER` column. This is useful when you want to automate ID generation for an existing table.

```sql
-- Original table
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT
);

-- Make the id column auto-incremental
ALTER TABLE users MODIFY COLUMN id INTEGER AUTOINCREMENT;

-- Now inserts don't need an explicit id
INSERT INTO users (name) VALUES ('Alice');
INSERT INTO users (name) VALUES ('Bob');

SELECT * FROM users;
-- Output will have id 1 and 2
```

## Adding UNIQUE constraints

You can ensure that all values in a column are unique by adding a `UNIQUE` constraint.

```sql
CREATE TABLE employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT
);

-- Enforce unique emails
ALTER TABLE employees MODIFY COLUMN email TEXT UNIQUE;

-- This will succeed
INSERT INTO employees (email) VALUES ('test@example.com');

-- This will fail with a unique constraint violation
INSERT INTO employees (email) VALUES ('test@example.com');
```

## Adding CHECK constraints

You can enforce data integrity rules by adding a `CHECK` constraint.

```sql
CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    price FLOAT
);

-- Ensure price is always positive
ALTER TABLE products MODIFY COLUMN price FLOAT CHECK (price > 0);

-- This will succeed
INSERT INTO products (price) VALUES (19.99);

-- This will fail with a check constraint violation
INSERT INTO products (price) VALUES (-5.00);
```
