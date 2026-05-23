# Quickstart: Fix Schema Name Bug

Once this fix is fully implemented, creating tables in specific schemas will correctly isolate them, avoiding the previous bug where they were dumped into the `public` schema with prepended names.

### Example Usage (Internal SQL Execution):

```sql
-- Creates the table 'cron_runs' strictly inside the 'system' namespace map
CREATE TABLE system.cron_runs (
    id INTEGER PRIMARY KEY
);

-- Can be queried properly across boundaries
SELECT * FROM system.cron_runs;

-- System catalog tables will reflect the proper namespace
SELECT schema_name, table_name FROM system.tables WHERE table_name = 'cron_runs';
-- Outputs: 
-- schema_name = 'system'
-- table_name  = 'cron_runs'
```