# Data Model & Interfaces: Context-Aware Autocomplete

## Internal Entities

### `SqlHelper`
- **Location**: `src/bin/oxibase.rs`
- **Fields**:
  - `db: Database`: A clone of the main database instance to run schema queries.
- **Responsibilities**: Implements the `rustyline::Helper` and related traits. Responsible for evaluating the current line text and returning autocomplete candidates.

### Autocomplete Candidates
- **SQL Keywords**: `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `CREATE`, `DROP`, `ALTER`, `TABLE`, `INDEX`, `VIEW`, `FROM`, `WHERE`, `JOIN`, `INNER`, `LEFT`, `RIGHT`, `ON`, `GROUP BY`, `ORDER BY`, `HAVING`, `LIMIT`, `OFFSET`, `INTO`, `VALUES`, `SET`, `BEGIN`, `COMMIT`, `ROLLBACK`.
- **CLI Commands**: `help`, `exit`, `quit`, `\q`, `\h`, `\?`.
- **Dynamic Objects**: Table names fetched dynamically from `information_schema.tables`.

## Interfaces (Contracts)
Since this feature modifies the internal CLI binary (`src/bin/oxibase.rs`), it does not expose new external API contracts, network endpoints, or change the CLI's command-line arguments.

The interaction is purely internal to the REPL prompt loop via the `rustyline::completion::Completer` interface:
```rust
fn complete(
    &self, 
    line: &str, 
    pos: usize, 
    ctx: &Context<'_>
) -> Result<(usize, Vec<Pair>), ReadlineError>
```
