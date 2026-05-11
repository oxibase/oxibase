# Checklist: Design & Architecture (Event Triggers)

**Purpose**: Unit tests for the technical design and architecture requirements.
**Created**: 2026-05-11

## Architecture Completeness

- [ ] CHK001 - Are the lifecycle and scoping rules for procedural engine instances explicitly documented for trigger execution? [Completeness, Plan §Unknowns]
- [ ] CHK002 - Is the AST representation for `CREATE TRIGGER` and `DROP TRIGGER` fully defined, including all required properties? [Completeness, Plan §Unknowns]
- [ ] CHK003 - Are the data types and storage structure for the `_sys_triggers` catalog table explicitly defined? [Completeness, Plan §Unknowns]
- [ ] CHK004 - Is the mechanism for caching triggers in-memory (e.g., `TriggerRegistry`) to avoid per-row catalog lookups fully specified? [Completeness, Plan §Unknowns]

## Data Flow & Memory Safety

- [ ] CHK005 - Is the exact mechanism for passing `NEW` and `OLD` row references into the procedural environments (Rhai, Boa, RustPython) documented? [Completeness, Plan §Unknowns]
- [ ] CHK006 - Does the row-passing mechanism explicitly satisfy the "Zero-Copy Unikernel memory efficiency" constraint? [Consistency, Plan §Technical Context]
- [ ] CHK007 - Are the specific techniques to ensure Rust memory safety (avoiding lifetime entanglement or dangling pointers) when exposing row pointers to dynamic scripts defined? [Completeness, Plan §Unknowns]

## Edge Cases & Error Handling

- [ ] CHK008 - Is the specific mechanism and default depth limit for preventing trigger recursion documented? [Completeness, Spec §Edge Cases]
- [ ] CHK009 - Is the behavior defined for when a trigger references a table that is subsequently dropped? [Completeness, Spec §Edge Cases]
- [ ] CHK010 - Are the exact mechanisms for how an unhandled exception in a trigger aborts the surrounding DML transaction documented? [Completeness, Spec §FR-005]
- [ ] CHK011 - Is the representation of missing state (e.g., `OLD` during `INSERT`, `NEW` during `DELETE`) explicitly defined for the procedural environments? [Completeness, Spec §Edge Cases]

## Performance Constraints

- [ ] CHK012 - Is the methodology for measuring the "less than a 5% performance regression" for base DML operations without triggers defined? [Measurability, Spec §SC-004]
- [ ] CHK013 - Are the specific areas where allocations (like `Vec` clones) must be avoided explicitly identified in the trigger execution path? [Clarity, Plan §Constitution Check]
