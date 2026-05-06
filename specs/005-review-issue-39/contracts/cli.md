# CLI Contract

The following extensions are made to the Oxibase CLI arguments (`Args` -> `Commands` enum in `src/bin/oxibase.rs`).

## New Commands

### `create-app`

Generates a new declarative application scaffold.

**Arguments:**
- `<name>` (Positional): The name of the application, which will be used as the root directory name.

**Behavior:**
- Creates the `<name>` directory.
- Aborts and returns an error if `<name>` already exists.
- Creates subdirectories: `data/`, `templates/`, `routes/`, `functions/`.
- Writes default boilerplate files into these subdirectories.
- Prints a success message instructing the user how to use the `seed` command.

### `seed`

Reads an application directory and deterministically loads it into the database.

**Arguments:**
- `<app_dir>` (Positional): The path to the application directory.
- `--db, -d <db_path>` (Option): The database connection string (e.g., `file:///target.db`). Defaults to `memory://` or requires an explicit argument if we want to enforce persistence.

**Behavior:**
- Validates the directory structure.
- Opens a single transaction.
- Initializes system schemas.
- Clears old routes and templates.
- Executes SQL files (`data/`).
- Inserts templates (`templates/`).
- Inserts routes (`routes/`).
- Inserts functions (`functions/`).
- Commits the transaction, or rolls back on any error.