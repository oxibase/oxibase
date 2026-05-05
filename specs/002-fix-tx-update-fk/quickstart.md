# Quickstart: Fix Transaction Updates with Foreign Keys

## Testing the Fix

You can verify the bug fix by running the sequence of SQL commands that previously triggered the error.

```sql
-- Start CLI
cargo run

-- In the CLI, execute the sequence:
CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    price FLOAT NOT NULL,
    category TEXT,
    in_stock BOOLEAN,
    created_at TIMESTAMP
);

INSERT INTO products (id, name, description, price, category, in_stock, created_at)
VALUES (1, 'Laptop', 'High-performance laptop with 16GB RAM', 1299.99, 'Electronics', TRUE, NOW());

BEGIN TRANSACTION;
UPDATE products SET price = price * 0.9 WHERE category = 'Electronics';
ROLLBACK;

CREATE TABLE categories (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT
);

INSERT INTO categories (id, name, description) VALUES (1, 'Electronics', 'Electronic devices and gadgets');

ALTER TABLE products ADD COLUMN category_id INTEGER;
UPDATE products SET category_id = 1 WHERE category = 'Electronics';

-- This should now succeed without the "uncommitted changes" error
ALTER TABLE products ADD CONSTRAINT fk_category FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL;
```

## Running Automated Tests

A new integration test will be added to ensure this doesn't regress.

```bash
make test
# OR
cargo nextest run --profile ci
```
