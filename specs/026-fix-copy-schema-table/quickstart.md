# Quickstart: COPY Schema-Qualified Table Syntax

This feature allows loading data directly into a schema-qualified table via the `COPY` statement.

## Usage

```sql
-- Assuming a schema named 'cdm' and a table 'concept' exist
COPY cdm.concept FROM 'path/to/data.csv' WITH (FORMAT CSV, HEADER TRUE);

-- Quoted identifiers are also supported
COPY "my schema"."my table" FROM 'path/to/data.csv' WITH (FORMAT CSV);

-- Existing syntax continues to work (defaults to standard schema)
COPY concept FROM 'path/to/data.csv' WITH (FORMAT CSV);
```