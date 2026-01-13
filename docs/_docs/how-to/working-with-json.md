---
layout: default
title: Working with JSON Data
parent: How-to Guides
nav_order: 1
---

# Working with JSON Data

This guide provides practical instructions for working with JSON data in Oxibase, including storing, querying, and manipulating JSON values in your applications.

## Storing JSON Data

### Creating Tables with JSON Columns

Start by defining tables that include JSON columns for flexible data storage:

```sql
CREATE TABLE products (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  attributes JSON
);

CREATE TABLE user_profiles (
  user_id INTEGER PRIMARY KEY,
  profile_data JSON NOT NULL,
  metadata JSON
);
```

### Inserting JSON Data

Insert JSON data directly using SQL literals:

```sql
-- Insert simple JSON objects
INSERT INTO products (id, name, attributes)
VALUES (1, 'Smartphone', '{"brand": "Example", "color": "black", "specs": {"ram": 8, "storage": 128}}');

INSERT INTO products (id, name, attributes)
VALUES (2, 'Headphones', '{"brand": "Example", "wireless": true, "battery_life": 24}');

-- Insert arrays
INSERT INTO user_profiles (user_id, profile_data, metadata)
VALUES (1, '{"name": "Alice", "tags": ["developer", "manager"]}', '{"version": 1, "last_updated": "2024-01-01"}');

-- Insert null values
INSERT INTO products (id, name, attributes)
VALUES (3, 'Basic Product', NULL);
```

### Using Parameterized Queries

For application code, use parameterized queries to safely insert JSON:

```sql
-- Using placeholders (implementation depends on your driver)
INSERT INTO products (id, name, attributes) VALUES (?, ?, ?);
-- Parameters: 4, 'Tablet', '{"brand": "Example", "screen_size": 10.5}'

INSERT INTO products (id, name, attributes) VALUES ($1, $2, $3);
-- Parameters: 5, 'Laptop', '{"specs": {"cpu": "i7", "ram": 16}}'
```

## Retrieving JSON Data

### Basic JSON Queries

Retrieve entire JSON columns or combine with other data:

```sql
-- Get all products with their JSON attributes
SELECT id, name, attributes FROM products;

-- Filter by non-JSON columns
SELECT id, attributes FROM products WHERE name = 'Smartphone';

-- Filter by JSON equality (basic comparison supported)
SELECT id FROM products
WHERE attributes = '{"brand": "Example", "color": "black", "specs": {"ram": 8, "storage": 128}}';
```

### Extracting Values from JSON

Use JSON functions to access specific parts of your JSON data:

```sql
-- Extract simple values
SELECT id, JSON_EXTRACT(attributes, '$.brand') AS brand
FROM products;

-- Extract nested values
SELECT id, JSON_EXTRACT(attributes, '$.specs.ram') AS ram_gb
FROM products
WHERE JSON_EXTRACT(attributes, '$.specs.ram') >= 8;

-- Extract array elements
SELECT user_id, JSON_EXTRACT(profile_data, '$.tags[0]') AS primary_tag
FROM user_profiles;

-- Get JSON type information
SELECT id,
       JSON_TYPE(attributes) AS json_type,
       JSON_KEYS(attributes) AS attribute_keys
FROM products;
```

## Updating JSON Data

### Replacing Entire JSON Values

Update complete JSON documents:

```sql
-- Update product specifications
UPDATE products
SET attributes = '{"brand": "Example", "color": "red", "specs": {"ram": 16, "storage": 256}}'
WHERE id = 1;

-- Update user profile data
UPDATE user_profiles
SET profile_data = '{"name": "Alice Johnson", "tags": ["developer", "manager", "team_lead"]}'
WHERE user_id = 1;
```

## Application Integration Examples

### Rust with Serde

Use `serde_json` for seamless JSON handling in Rust applications:

```rust
use oxibase::Database;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ProductAttributes {
    brand: String,
    color: Option<String>,
    specs: Option<ProductSpecs>,
}

#[derive(Serialize, Deserialize)]
struct ProductSpecs {
    ram: u32,
    storage: u32,
}

#[derive(Serialize, Deserialize)]
struct UserProfile {
    name: String,
    tags: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("memory://")?;

    // Create tables
    db.execute(
        "CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, attributes JSON)",
        ()
    )?;
    db.execute(
        "CREATE TABLE user_profiles (user_id INTEGER PRIMARY KEY, profile_data JSON)",
        ()
    )?;

    // Insert data using Rust structs
    let product = ProductAttributes {
        brand: "Example".to_string(),
        color: Some("blue".to_string()),
        specs: Some(ProductSpecs { ram: 16, storage: 512 }),
    };
    let product_json = serde_json::to_string(&product)?;
    db.execute(
        "INSERT INTO products (id, name, attributes) VALUES (?, ?, ?)",
        (1, "Laptop", &product_json)
    )?;

    // Query and deserialize
    if let Some(row) = db.query("SELECT attributes FROM products WHERE id = ?", (1,))?.next() {
        let row = row?;
        let attributes_json: String = row.get("attributes")?;
        let parsed: ProductAttributes = serde_json::from_str(&attributes_json)?;
        println!("Brand: {}", parsed.brand);
    }

    // Insert user profile
    let profile = UserProfile {
        name: "Alice".to_string(),
        tags: vec!["developer".to_string(), "rust".to_string()],
    };
    let profile_json = serde_json::to_string(&profile)?;
    db.execute(
        "INSERT INTO user_profiles (user_id, profile_data) VALUES (?, ?)",
        (1, &profile_json)
    )?;

    Ok(())
}
```

### JavaScript/Node.js

Handle JSON data in JavaScript applications:

```javascript
const oxibase = require('oxibase');

// Connect to database
const db = oxibase.connect('file:database.db');

// Create table
db.exec(`
  CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT,
    attributes JSON
  )
`);

// Insert JSON data
const product = {
  brand: 'Example',
  specs: { ram: 8, storage: 128 },
  features: ['wireless', 'fast charging']
};

db.run(
  'INSERT INTO products (id, name, attributes) VALUES (?, ?, ?)',
  [1, 'Phone', JSON.stringify(product)]
);

// Query with JSON extraction
const rows = db.all(`
  SELECT
    id,
    name,
    JSON_EXTRACT(attributes, '$.brand') as brand,
    JSON_EXTRACT(attributes, '$.specs.ram') as ram
  FROM products
  WHERE JSON_EXTRACT(attributes, '$.specs.ram') >= 8
`);

console.log(rows);
// Output: [{ id: 1, name: 'Phone', brand: 'Example', ram: 8 }]
```

## Advanced JSON Queries

### Complex Filtering

Use JSON functions for sophisticated queries:

```sql
-- Find products with specific features
SELECT id, name
FROM products
WHERE JSON_EXTRACT(attributes, '$.features') LIKE '%wireless%';

-- Check for existence of keys
SELECT id, name
FROM products
WHERE JSON_KEYS(attributes) LIKE '%"specs"%';

-- Validate JSON before processing
SELECT id, attributes
FROM products
WHERE JSON_VALID(attributes) = 1;
```

### Aggregating JSON Data

Combine JSON extraction with aggregation:

```sql
-- Count products by brand
SELECT
  JSON_EXTRACT(attributes, '$.brand') AS brand,
  COUNT(*) AS product_count
FROM products
GROUP BY JSON_EXTRACT(attributes, '$.brand');

-- Find average specs across products
SELECT
  AVG(JSON_EXTRACT(attributes, '$.specs.ram')) AS avg_ram,
  MAX(JSON_EXTRACT(attributes, '$.specs.storage')) AS max_storage
FROM products
WHERE JSON_TYPE(JSON_EXTRACT(attributes, '$.specs')) = 'object';
```

## Best Practices

### Schema Design

- **Hybrid approach**: Store frequently queried fields as regular columns, use JSON for flexible metadata
- **Validate upfront**: Always validate JSON structure in your application before database insertion
- **Keep reasonable sizes**: JSON documents should be reasonably sized for performance

### Query Optimization

- **Extract once**: Store frequently accessed JSON values in separate columns if performance is critical
- **Index strategically**: Consider partial indexes on JSON-extracted values for large datasets
- **Type checking**: Use JSON_TYPE() to ensure expected data types before operations

### Application Patterns

- **Serialization**: Use native JSON serialization in your programming language
- **Error handling**: Implement proper error handling for JSON parsing and validation
- **Versioning**: Consider versioning schemes for JSON structures that may evolve

## Complete Example

Here's a comprehensive example showing JSON usage in a product catalog:

```sql
-- Create the database schema
CREATE TABLE products (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  category TEXT,
  price DECIMAL(10,2),
  attributes JSON,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample data
INSERT INTO products (id, name, category, price, attributes) VALUES
(1, 'MacBook Pro 16"', 'Laptops', 2499.00,
 '{"brand": "Apple", "specs": {"cpu": "M2", "ram": 32, "storage": 1024}, "features": ["retina", "touchbar"]}'),
(2, 'Dell XPS 13', 'Laptops', 1299.00,
 '{"brand": "Dell", "specs": {"cpu": "i7", "ram": 16, "storage": 512}, "features": ["ultrabook", "infinityedge"]}'),
(3, 'iPhone 15 Pro', 'Phones', 999.00,
 '{"brand": "Apple", "specs": {"storage": 256, "camera": "48MP"}, "features": ["wireless", "5g", "titanium"]}');

-- Query examples
-- Find all Apple products
SELECT id, name, price
FROM products
WHERE JSON_EXTRACT(attributes, '$.brand') = 'Apple';

-- Find products with at least 16GB RAM
SELECT name, JSON_EXTRACT(attributes, '$.specs.ram') AS ram_gb
FROM products
WHERE JSON_EXTRACT(attributes, '$.specs.ram') >= 16;

-- Get product features as a list
SELECT name,
       JSON_EXTRACT(attributes, '$.features') AS features
FROM products
WHERE JSON_TYPE(JSON_EXTRACT(attributes, '$.features')) = 'array';

-- Advanced search: products with specific features
SELECT name, category
FROM products
WHERE JSON_EXTRACT(attributes, '$.features') LIKE '%wireless%';
```

This approach gives you the flexibility of JSON storage while maintaining the
performance and reliability of relational data where it matters most.

## Next Steps

- Check the reference documentation for the [JSON] type.
- Explore [JSON functions] for data manipulation and aggregation.


[JSON]: {% link _docs/references/data-types.md %}
[JSON functions]: {% link _docs/references/functions/scalar-functions.md %}
