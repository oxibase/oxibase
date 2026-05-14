# OpenCode Agent Instructions

This file contains high-signal, repo-specific context to help agents work effectively in the `oxibase` codebase.

## Development Commands
- **Testing**: Use `cargo nextest run` instead of `cargo test`. 
  - Standard tests: `make test` (or `cargo nextest run --profile ci`)
  - All features: `make test-all`
- **Linting & Formatting**: Use `make lint` (runs `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`).
- **Features**: The codebase has optional scripting backends: `js` (Boa), `python` (RustPython), and `rhai` (default). When testing or building specific backend logic, ensure you pass the right feature flag (e.g., `--features js`).
- **License Headers**: All `.rs` files require an Apache-2.0 license header. You can use `./scripts/fix_copyrights.sh` to fix headers or run `make license` to verify them. 

## Code Standards
- **No `unwrap()`**: Avoid using `unwrap()` or `expect()` in library code. Propagate errors properly using `Result` and the `thiserror`/`anyhow` crates.
- **No `todo!()` or `unimplemented!()`**: Do not commit incomplete code.
- **Unsafe**: Avoid `unsafe` code unless strictly necessary. If used, it must be thoroughly documented.

## Architecture & Entrypoints
The system is an autonomous relational database management system.
- `src/lib.rs` / `src/api/database.rs`: Main entry points for the library. `Database` wraps the storage engine.
- `src/bin/oxibase.rs`: The CLI binary entry point (requires `cli` feature).
- `src/parser/`: SQL lexer, AST, and parser.
- `src/optimizer/`: Cost-based query optimizer and workload-based optimization.
- `src/executor/`: Query execution engine (interprets AST into execution).
- `src/storage/`: Storage engine, transactions, and MVCC (Multi-Version Concurrency Control).
- `src/functions/`: Built-in scalar, aggregate, and window functions.
- `src/core/`: Core data structures (`Value`, `Row`, `Schema`, `Error`).

## CI/CD & Artifacts
- CI automatically runs `make lint`, `cargo nextest` across features (`js`, `python`), and checks licenses.
- Code coverage is uploaded to Codecov via `cargo llvm-cov` (`make coverage`).
- Binary artifacts are built for Linux, macOS, and Windows on release. Ensure any new dependencies support cross-compilation on these targets.

<!-- SPECKIT START -->
/Users/gabriel.maeztu/repos/oxibase/specs/014-job-scheduler/plan.md
<!-- SPECKIT END -->

