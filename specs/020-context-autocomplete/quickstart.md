# Quickstart: Testing Context-Aware Autocomplete

To test the context-aware autocomplete in the CLI:

1. **Build and Run the CLI**:
   ```bash
   cargo run --features cli
   ```

2. **Test Keyword Autocomplete**:
   At the `>` prompt, type `SEL` and hit `<Tab>`. It should complete to `SELECT `.
   Type `CREA` and hit `<Tab>`. It should complete to `CREATE `.

3. **Test Context-Aware Schema Autocomplete**:
   Create a table:
   ```sql
   > CREATE TABLE my_awesome_table (id INTEGER);
   ```

   Now try autocompleting the table name. Type:
   ```sql
   > SELECT * FROM my_a<Tab>
   ```
   It should complete to `my_awesome_table `.

   Try another context like `INSERT INTO`:
   ```sql
   > INSERT INTO my_<Tab>
   ```
   It should complete to `my_awesome_table `.

4. **Verify Performance**:
   There should be no noticeable lag when hitting `<Tab>`, even when it triggers a database query.
