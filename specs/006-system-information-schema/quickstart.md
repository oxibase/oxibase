# System Metadata Quickstart

Oxibase exposes its internal database metadata through standard SQL interfaces. This allows standard database tools (like DBeaver or DataGrip) to introspect the schema, and provides administrators with deep visibility into system internals.

## Using `information_schema` (Public Standard)

The `information_schema` is a standard SQL schema consisting of read-only views. It is the recommended way for external tools and standard users to query the database structure.

**List all tables and views:**
```sql
SELECT table_schema, table_name, table_type 
FROM information_schema.tables;
```

**View column definitions for a specific table:**
```sql
SELECT column_name, data_type, is_nullable, column_default 
FROM information_schema.columns 
WHERE table_name = 'users';
```

## Using `system` Schema (Internal Storage)

The `system` schema is the physical storage layer for oxibase's metadata. Unlike `information_schema` (which formats data to standard SQL specifications), the `system` tables store the raw internal representations.

**Query raw table definitions:**
```sql
SELECT id, schema_name, table_name, created_at
FROM system.tables;
```

**Query raw column properties:**
```sql
SELECT id, table_id, column_name, ordinal_position, is_primary_key
FROM system.columns;
```

## Restrictions

- **Read-Only**: Both `information_schema` and `system` schemas are strictly read-only for standard queries. You cannot execute `INSERT`, `UPDATE`, or `DELETE` against tables within these schemas.
- **Modifying Metadata**: To modify the metadata, use standard Data Definition Language (DDL) commands (e.g., `CREATE TABLE`, `ALTER TABLE`, `DROP TABLE`). The oxibase engine automatically translates these DDL commands into the appropriate transactional updates against the `system` schema.