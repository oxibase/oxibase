---
title: Home
layout: default
nav_exclude: true
---

<div style="text-align: center;"><img src="assets/img/logo.svg" alt="Oxibase Logo" style="max-width: 200px; height: auto;"></div>

Oxibase is a relation database operating system (DBOS) that provides full ACID
transactions with MVCC and a sophisticated cost-based query optimizer. It
leverages unikernel technology for kernel-integrated performance, to enable
privileged hardware access and zero-copy data paths.

- **Multiple Index Types**: B-tree, Hash, and Bitmap indexes with automatic type selection
- **Multi-Column Indexes**: Composite indexes for complex query patterns
- **Parallel Query Execution**: Automatic parallelization using Rayon for large datasets
- **Cost-Based Optimizer**: PostgreSQL-style optimizer with adaptive execution and cardinality feedback
- **Semantic Query Caching**: Intelligent result caching with predicate subsumption
- **Disk Persistence**: WAL and snapshots with crash recovery
- **Rich SQL Support**: Window functions, CTEs (including recursive), subqueries, ROLLUP/CUBE, and 101+ built-in functions
- **User-Defined Functions**: Extend SQL with custom functions written in Rhai, Python, or JavaScript

## Roadmap Overview

Oxibase is evolving from an embedded database into a distributed, autonomous database system that bridges embedded logic with global resilience. The roadmap outlines the journey from a single-node, scriptable database to a self-managing, platform.

**Phase 1: Foundation** - Establishing core capabilities through parallel efforts: embedded scripting languages for stored functions and triggers, declarative schema migrations, TUI manager, web server exposure for REST/GraphQL, Postgres wire protocol server, deterministic simulator, and unikernel compilation with kernel integration benefits like zero-copy data paths and privileged hardware access. These prepare for horizontal scaling.

**Phase 2: Single Node Efficiency** - Unifying operational needs into Oxibase itself with self-monitoring, multiple storage backends, data lifecycle management, and separation of compute & storage.

**Phase 3: Distributed System** - Achieving infinite horizontal scale and global resilience through multi-master consensus, data rebalancing/sharding, geo-sharding, and gossip protocols.

**Phase 4: The Autonomous Cloud** - The database manages its own physical existence and evolves into an autonomous platform with networking, auto-infrastructure management, in-database ML inference, and GPU training.

_Figure 1:  Dependency Flow_

```mermaid
classDiagram
    %% --- RELATIONSHIPS ---
    %% Use the short class names here, not "Layer.Class"
    Computational_Layer ..> Web_Interface
    Computational_Layer ..> External_Gateway
    Computational_Layer ..> Performance

    Web_Interface ..> Horizontal_Architecture
    External_Gateway ..> Horizontal_Architecture
    Performance ..> Horizontal_Architecture
    Unikernel_Compiler ..> Horizontal_Architecture

    %% --- LAYERS & CLASS DEFINITIONS ---

    class Computational_Layer {
        - Embedded Scripting Languages
        - Stored functions
        - Triggers
        - Queues
        - Debugger support
        - FaaS-like DevEx
        - Out-of-core processing
    }
    class Web_Interface {
        - DML routes
        - REST / GraphQL endpoints
        - HTML Render
    }
    class External_Gateway {
        - Postgres Wire Protocol Server
        - Role based authorization control
        - Authentication
    }

    class Performance {
        - Vertical scaling support
        - Out-of-core processing
        - Self-monitoring
        - Undo-log based MVCC
        - Copy-on-write checkopoint
        - Row level replication
        - Storage backends
    }

    class Horizontal_Architecture {
        - Deterministic Simulator
        - Failure simulation
        - Logic and storage separation
        - Dedicated Clustering
        - Sharding
        - Geolocation
        - Distributed Computation
        - Data / Computation rebalancing
    }

    class Unikernel_Compiler {
        - Unikernel Compilation
        - OS-free images
        - Bootable image generation
        - SQL based OS management
    }

    class Kernel_Integration {
        - Zero-copy data paths
        - Privileged hardware access
        - Lock-free page-table walks
        - Active virtual memory
        - Elastic resource allocation
    }

    Unikernel_Compiler ..> Kernel_Integration
    Kernel_Integration ..> Horizontal_Architecture

```

For detailed information about each phase, see the [full roadmap](getting-started/roadmap).

## Need Help?

If you can't find what you're looking for in the documentation, you can:

- [Open an issue](https://github.com/oxibase/oxibase/issues) on GitHub
- [Join the discussions](https://github.com/oxibase/oxibase/discussions) to ask questions

---

This documentation is under active development. Contributions are welcome!
