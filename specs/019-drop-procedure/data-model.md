# Data Model: Drop Procedure

No new data entities are being created. We are utilizing the existing `pg_proc` system catalog.

## Entities Involved
*   **System Catalog (`pg_proc`)**: Contains procedure definitions. `DROP PROCEDURE` will delete a row from this table.

## Changes
*   None to the data schema.
