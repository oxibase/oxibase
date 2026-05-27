# Research: Rename Routes and Templates Interfaces

## Context
The feature specification requires renaming `routes.definitions` to `interface.routes` and `templates.sources` (which currently exists as `templates.source` in the codebase) to `interface.templates`.

## Findings
- **`routes.definitions`**: Appears primarily in `src/bin/workspace/mod.rs`, `src/server/handlers.rs`, `src/server/mod.rs`, and `tests/server_test.rs` inside raw SQL queries used for schema initialization and routing.
- **`templates.source`**: Appears primarily in `src/bin/workspace/mod.rs`, `src/server/mod.rs`, `src/server/template.rs`, `tests/server_test.rs`, and examples. The user spec referenced `templates.sources`, but we must update the actual `templates.source` references in the codebase.

## Decisions
- **Decision**: Perform a codebase-wide find-and-replace for `routes.definitions` -> `interface.routes` and `templates.source` -> `interface.templates`.
- **Rationale**: This is a direct name refactoring task. The names are entirely contained within SQL strings and documentation.
- **Alternatives considered**: None, as this is a direct user request to rename specific strings.
