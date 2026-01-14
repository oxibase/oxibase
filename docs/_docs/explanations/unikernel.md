---
layout: default
title: "Unikernel"
parent: Explanations
nav_order: 3
---

## Kernel Integration Benefits

OxiBase builds on research from CumulusDB{% cite unikernels %}, a
unikernel-based DBMS that integrates kernel primitives for optimal performance,
inspired by related work on OS-DBMS co-design{% cite cod2013 %}, virtual memory
snapshots{% cite hyper2011 %}, buffer management{% cite buffer2023 %}, and
OS-oriented systems{% cite dbos_progress2022 %}{% cite skiado2021 %}. Kernel
integration provides privileged access to hardware and OS resources, enabling
features that traditional layered architectures cannot achieve.

### Privileged Hardware Access
- **Direct NVMe Queue Access**: Bypass OS storage layers for kernel-bypass I/O,
  reducing latency in high-performance SSD scenarios.
- **Lock-Free Page-Table Walks**: Concurrent, lock-free access to MMU structures
  for fast virtual-page presence checks (demonstrated 40x speedup in CumulusDB
  benchmarks{% cite unikernels %}).
- **IRQ and CPU Management**: Manipulate interrupt vectors, block IRQs, and
  control CPU shutdown for query-plan-aware scheduling.

### Zero-Copy Data Paths
- Stream data from query execution to network results without copying,
  leveraging hypervisor-shared memory regions{% cite unikernels %}.
- Elastic resource allocation through hypercalls for dynamic VM scaling{% cite
  unikernels %}.

### Active Virtual Memory Management
- Virtual memory as an active abstraction, with inconsistent TLB states handled
  efficiently for concurrent OLTP/OLAP workloads{% cite unikernels %}.
- Advanced snapshots using ad-hoc parallelization and reader-side TLB
  invalidation, avoiding TLB shootdowns{% cite sharma2018 %}, with evaluation of
  virtual memory primitives{% cite loeck2023 %} and support for heterogeneous
  hardware{% cite muhlig2020 %}.

These benefits align with OxiBase's unikernel compilation goals, enabling full
hardware exploitation without OS overhead. See the [roadmap]({% link
_docs/roadmap.md %}) for more details.
