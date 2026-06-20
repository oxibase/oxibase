# Research: AST-in-AST Static Analysis for Related Objects Detection

This document details the architectural decisions and research conducted for supporting native AST-in-AST static analysis across scripting backends in Oxibase.

## Decisions & Designs

### Decision 1: SQL AST Visitor Implementation
- **Decision**: Implement a custom `Visitor` trait and `walk_statement` / `walk_expression` helper functions in `src/parser/visitor.rs`.
- **Rationale**: Keeps the core parsing module decoupled from specific analysis use cases. A generic Visitor trait allows other parts of the database (like optimization or security modules) to traverse the SQL AST without rewriting walking boilerplate.
- **Alternatives Considered**: Direct manual pattern matching inside `DependencyExtractor`. Rejected because it would mix traversal boilerplate with specific collection logic, making the code harder to read and maintain.

### Decision 2: Rhai AST Walking via `internals` Feature
- **Decision**: Walk `rhai::AST` using direct recursion over its statements and user-defined functions.
- **Rationale**: Since `rhai` internals are fully enabled in `Cargo.toml`, we can traverse `Stmt` and `Expr` directly. Walking user-defined functions (`ast.iter_functions()`) as well as top-level statements ensures no database call is missed.
- **Alternatives Considered**: Running a regex search on the script. Rejected because regexes are extremely fragile, error-prone, and cannot accurately identify the context of database calls (e.g. comments, string formatting, scopes).

### Decision 3: Python AST Walking via `rustpython-vm`
- **Decision**: Parse the Python script using `rustpython_vm::compiler::parser::parse(..., Mode::Exec, ...)` and walk the resulting statements and expressions recursively.
- **Rationale**: Leverages the official compiler and parser already bundled with the database's RustPython engine. This guarantees 100% correct parsing behavior matching the database's actual Python execution runtime.
- **Alternatives Considered**: Integrating an external Python parsing crate. Rejected because it would add a redundant dependency, violating our dependency and monolith principles.

### Decision 4: Graceful Dynamic Query Handling
- **Decision**: Identify variable SQL strings (such as string concatenations, function results, or variable bindings) in the AST as non-literal nodes, and gracefully report them using a `"Dynamic"` object marker.
- **Rationale**: Statically determining dynamic queries is theoretically impossible (equivalent to the halting problem). Adding a `"Dynamic"` marker informs the TUI/CLI that there are run-time dependencies that couldn't be statically resolved, which is highly informative and safe.
