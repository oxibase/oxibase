---
title: Home
layout: default
nav_exclude: true
---

<div style="text-align: center;"><img src="assets/img/logo.svg" alt="Oxibase Logo" style="max-width: 200px; height: auto;"></div>

Oxibase is an autonomous relational database management operating system (DBMOS)
written in Rust that embodies a _"Modern Mainframe"_ philosophy: bringing
computation directly to data. The system provides ACID-compliant transactions
with multi-version concurrency control (MVCC), supporting both in-memory and
persistent operation modes.

### **Core Philosophy:** 

Oxibase rejects the traditional separation of "application server" and "database
server" as an artifact of historical hardware constraints. By co-locating
computation and data, the system eliminates network latency and serialization
overhead inherent in distributed architectures. This "computation to data"
approach enables user-defined functions in multiple languages to execute within
transaction scope, directly where data resides.

### **Current State:** 

An embedded SQL database with 100+ built-in functions, three index types
(B-tree, Hash, Bitmap), cost-based query optimization, and advanced SQL features
including window functions, recursive CTEs, and time-travel queries.

### **Future Vision:** 

Evolution toward a distributed unikernel-based system with kernel-integrated
performance, embedding a multiple computational paradigms, autonomous scaling,
and in-database machine learning. The roadmap progresses from embedded scripting
(Phase 1) through distributed consensus (Phase 3) to self-managing
infrastructure with GPU-accelerated ML inference (Phase 4).


## Project Goals

- **Self-sufficiency:** Oxibase aspires to be a fully self-contained system,
  minimizing external dependencies for both development and deployment. The
  database should provide everything needed—compute, storage, logic, and
  orchestration—within a cohesive environment.
- **Strong Opinions:** The architecture and feature set are intentionally
  opinionated, favoring bold, clear principles over generic extensibility.
  Decisions are made for users to reduce ambiguity and increase focus.
- **Learning & Research:** Oxibase is a playground for exploring new ideas in
  database systems, distributed architectures, transactionality, and co-location
  of data and logic. Continuous learning and disseminating insights are core to
  the project.
- **Heavily Tested:** Reliability and correctness matter deeply. Features and
  infrastructure are expected to be exhaustively tested.
- **Accessible for Humans:** Readability and clarity of code, configuration, and
  operation are prioritized—even at the expense of some automation or
  performance. The system should be understandable by curious practitioners.

## Explicit Non-Goals

- **Maximum Performance:** Raw benchmark performance is not the primary pursuit.
  Reasonable performance is required, but clarity and correctness take
  precedence.
- **Strict Standards Conformance:** While best effort will be made for
  compatibility (e.g., SQL, network protocols), strict adherence to industry
  standards is not a goal. Deviations may be made for clarity, simplicity, or
  research motivations.
- **Prioritizing Automation Over Clarity:** Design choices that favor ease of
  maintenance, modification, or explanation—even if that leads to less
  automation or a “bottleneck” for throughput—will be preferred.
- **Generic Extensibility:** Oxibase is explicitly not “one size fits all.” It
  targets specific philosophies and refuses to chase universal flexibility.

For detailed information about each phase, see the [full roadmap]({% link
_docs/roadmap.md %}).

## New to Oxibase?

If you're new to Oxibase, we recommend starting with our [Quickstart Guide]({% link
_docs/getting-started/quickstart.md %}) to get up and running quickly.

## Need Help?

If you can't find what you're looking for in the documentation, you can:

- [Open an issue](https://github.com/oxibase/oxibase/issues) on GitHub
- [Join the discussions](https://github.com/oxibase/oxibase/discussions) to ask
  questions

---

This documentation is under active development. Please check back regularly for updates.
