---
title: Home
layout: default
nav_exclude: true
---

<div style="text-align: center;"><img src="assets/img/logo.svg" alt="Oxibase Logo" style="max-width: 200px; height: auto;"></div>

Oxibase is an autonomous database management operating system (DBMOS) that
embodies a _"Modern Mainframe"_ philosophy: bringing computation directly to
data. It aims to become a self-contained computational runtime, providing a
complete solution for data management and computation.

### **Core Philosophy:** 

Oxibase rejects the traditional separation of _"application server"_ and
_"database server"_ as an artifact of historical hardware constraints. By
co-locating computation and data, the system eliminates complexity, network
latency and serialization overhead inherent in distributed architectures. This
_"computation to data"_ approach enables user-defined business logic in multiple
languages and paradigms to execute within transaction scope, directly/close to
where data resides thanks to data locality awareness of the system.

### **Current State:** 

An embedded SQL database with three index types (B-tree, Hash, Bitmap),
cost-based query optimization, built-in functions, user-defined functions, and
advanced SQL features including window functions, recursive CTEs, and
time-travel queries.

### **Future Vision:** 

The goal is evolution toward a distributed computational runtime in a
unikernel-based system {% cite unikernels %} that embeds a multiple
computational paradigms for fast development and auto-scaling. It aims to
revisit some of the ideas behind IBM System i {% cite IBM_i %} with a
fresh and open-source approach. The goal is to provide storage (everything is a
relational object), business-logic (SQL, JS, Python and Rhai), API (REST, gRPC,
WebSocket), orchestration, self-monitoring, self-tracing, version-control,
authorization, authentication, and encryption through a unified interface in the
form of Relational Database Management System (RDBMS). 

The project roadmap outlines a strategic progression, starting with the
development of the core experience (Phase 0), as an small embedded system with
the goal of providing a self-contained computational experience, providing a
comprehensive environment for both development and deployment. The system should
provide support to store data, store and execute procedures, debug them, monitor
and trace them. This should serve as a demo of the system's capabilities, to
understand if the DevEx is viable.

For detailed information about each phase, see the [full roadmap]({% link
_docs/roadmap.md %}).

## Project Goals

- **Self-sufficiency:** Oxibase aspires to be a fully self-contained
  computational runtime, providing a comprehensive environment for both
  development and deployment. The system should provide everything needed within
  a cohesive environment. 
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
