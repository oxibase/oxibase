# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2025-12-27

### Added
- Added cargo-nextest for faster test execution
- Added diagrams and mermaid zoom functionality to documentation
- Added Makefile to manage development workflow
- Updated documentation for research emphasis and roadmap alignment

### Fixed
- Fixed indentation for 'Run tests' step in CI workflow

## [0.1.1] - 2025-12-20

### Fixed
- SUBSTRING function test now correctly expects "OXI" instead of "STO" for SUBSTRING('OXIBASE', 1, 3)

## [0.1.0] - 2025-12-20

### Added
- Initial release of OxiBase as an embedded SQL database
- Full MVCC (Multi-Version Concurrency Control) with ACID compliance
- Support for in-memory and persistent storage modes
- Comprehensive SQL support including DDL, DML, and queries
- Built-in functions: 100+ scalar, aggregate, and window functions
- Multiple index types: B-tree, Hash, Bitmap
- Time-travel queries with AS OF clauses
- Window functions (ROW_NUMBER, RANK, etc.)
- Common Table Expressions (CTEs) including recursive queries
- Advanced aggregations with ROLLUP, CUBE, GROUPING SETS
- Query optimizer with cost-based planning and EXPLAIN
- Persistence with WAL (Write-Ahead Logging) and snapshots
- Command-line interface for REPL and query execution
- Rust API for embedding in applications

### Features
- Transactions with Read Committed and Snapshot isolation levels
- Data types: INTEGER, FLOAT, TEXT, BOOLEAN, TIMESTAMP, JSON
- Subqueries (scalar, correlated, EXISTS, IN)
- Full-text search capabilities
- JSON data manipulation
- Date/time functions and operations
- Parallel query execution
- Semantic and query caching

This release establishes the foundation for future evolution into a unikernel-based mainframe architecture.