# Research: Context-Aware Autocomplete

## Decision: Table Name Fetching
- **Decision**: Query the `information_schema.tables` virtual table.
- **Rationale**: The executor already uses `SELECT table_name FROM information_schema.tables WHERE table_schema != 'system' OR table_schema IS NULL` for the `SHOW TABLES` command. This is an existing, fast, and idiomatic way to get a list of user tables without diving into internal engine APIs from the CLI.
- **Alternatives considered**: Querying internal engine metadata (e.g., `db.engine().get_all_tables()`). Rejected because we want to maintain the `Database` API abstraction layer in the CLI when possible.

## Decision: Rustyline Completer Implementation
- **Decision**: Implement a custom `SqlHelper` struct that holds a clone of `Database` and implements the `rustyline::completion::Completer`, `rustyline::Helper`, `rustyline::hint::Hinter`, `rustyline::highlight::Highlighter`, and `rustyline::validate::Validator` traits.
- **Rationale**: Rustyline requires a struct implementing `Helper` (which implies all the other traits) to provide autocompletions. `Database` is cheaply cloneable (`Arc` internally), so giving the helper its own reference to `Database` is memory efficient and safe.
- **Alternatives considered**: Using rustyline's default basic file path completer. Rejected because it does not provide SQL or database context.

## Decision: Context-Awareness Strategy
- **Decision**: Parse the current line up to the cursor to determine context. If the previous word indicates a table context (e.g., `FROM`, `INTO`, `UPDATE`, `JOIN`, `TABLE`), trigger a database query to fetch table names and filter them by the current word prefix. Otherwise, fallback to standard SQL keyword and CLI command completion.
- **Rationale**: Simple string-splitting logic is fast enough for interactive typing (needs to be < 50ms). Doing full SQL AST parsing on every keystroke in `rustyline` could be too slow and would break on incomplete SQL lines (which is exactly what the user is typing).
- **Alternatives considered**: Use the `parser::Parser` to parse the partial AST. Rejected because the parser generally returns an error on incomplete statements, making it hard to extract the exact autocomplete intent without a specialized "tolerant" parser.
