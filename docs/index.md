---
title: Home
layout: default
nav_exclude: true
---

<div style="text-align: center;"><img src="assets/img/logo.svg" alt="Oxibase Logo" style="max-width: 200px; height: auto;"></div>


OxiBase is a relation database operating system (DBOS) that provides full ACID transactions with MVCC, a sophisticated cost-based query optimizer.

- **Multiple Index Types**: B-tree, Hash, and Bitmap indexes with automatic type selection
- **Multi-Column Indexes**: Composite indexes for complex query patterns
- **Parallel Query Execution**: Automatic parallelization using Rayon for large datasets
- **Cost-Based Optimizer**: PostgreSQL-style optimizer with adaptive execution and cardinality feedback
- **Semantic Query Caching**: Intelligent result caching with predicate subsumption
- **Disk Persistence**: WAL and snapshots with crash recovery
- **Rich SQL Support**: Window functions, CTEs (including recursive), subqueries, ROLLUP/CUBE, and 101+ built-in functions


## Need Help?

If you can't find what you're looking for in the documentation, you can:
* [Open an issue](https://github.com/oxibase/oxibase/issues) on GitHub
* [Join the discussions](https://github.com/oxibase/oxibase/discussions) to ask questions

---

This documentation is under active development. Contributions are welcome!
