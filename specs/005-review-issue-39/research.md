# Research: App Scaffolding and Seeding CLI Commands

## Core Unknowns & Decisions

### 1. `clap` Subcommand Integration
- **Decision**: Add `CreateApp` and `Seed` subcommands to the `Commands` enum in `src/bin/oxibase.rs`.
- **Rationale**: `clap` is the standard arguments parser in this codebase.
- **Alternatives Considered**: Using a separate binary, which violates the monolithic design principle.

### 2. Transaction Management for `seed`
- **Decision**: The entire `seed` operation will be wrapped in a transaction using `db.begin()`, executing all schema creations, data loading, templates, routes, and functions inserts, and then `tx.commit()`.
- **Rationale**: Ensures the database doesn't end up in an inconsistent state if an error occurs during seeding (e.g., malformed route JSON or syntax error in a SQL script). This aligns with FR-004.
- **Alternatives Considered**: Auto-commit per file. Rejected because it risks partial app initialization.

### 3. File System Operations
- **Decision**: Use `std::fs::create_dir_all` and `std::fs::write` for scaffolding. For seeding, use `std::fs::read_dir` combined with collecting and sorting file names alphabetically (to satisfy FR-007 for SQL scripts).
- **Rationale**: Built-in standard library tools are adequate and do not require external dependencies. 

### 4. Overwrite Behavior (`create-app`)
- **Decision**: `create-app` will use `std::fs::metadata` to check if the directory exists. If it does, it will immediately return an error via `thiserror/anyhow` or standard `Result` mechanism.
- **Rationale**: Directly answers the user-clarified FR-011 (Abort if directory already exists).
