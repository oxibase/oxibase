# Quickstart: Internal Logging

Once implemented, you can test the feature with:

1. **Start the database in memory with verbose logging**:
   ```bash
   RUST_LOG=debug cargo run --features cli -- repl -d memory://
   ```

2. **Console Output**:
   You should see JSON-formatted logs printed to the terminal, confirming the stdout layer is working.

3. **Trigger an Error or Warning**:
   Execute a query that logs an INFO or WARNING internally. For example, executing a bad query or forcing a table creation.
   ```sql
   CREATE TABLE test (id INT);
   ```

4. **Query System Logs**:
   Read the logs back from the internal database table.
   ```sql
   SELECT * FROM system.logs ORDER BY timestamp DESC LIMIT 5;
   ```

5. **Verify No Hangs**:
   Ensure the database remains responsive, confirming the background flusher thread isn't caught in a recursive loop while inserting records.
