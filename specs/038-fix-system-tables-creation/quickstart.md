# Quickstart: Fix System Tables Creation

Because this is a low-level engine initialization fix, there are no new external APIs. However, to verify it works locally:

1. Delete any existing database state directory if applicable.
2. Start the database instance.
3. Connect and execute:
   ```sql
   -- Verify the tables exist
   SELECT schema_name, table_name FROM system.tables;
   
   -- Insert duplicate data to verify UNIQUE constraint failures
   ```