# Feature Specification: DAP Support for PL/SQL Procedures

**Feature Branch**: `039-dap-plsql`  
**Created**: 2026-06-11  
**Status**: Draft  
**Input**: User description: "for task #53"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently via `cargo nextest`
  - Deployed independently
-->

### User Story 1 - Attach and Pause at Breakpoint (Priority: P1)

As a database developer writing PL/SQL procedures in Oxibase, I want to be able to connect my IDE (VS Code, Zed, etc.) via the Debug Adapter Protocol (DAP) and set breakpoints in my native PL/SQL code, so that execution pauses precisely at the lines I specify, allowing me to inspect the flow of execution.

**Why this priority**: Pausing execution at a designated point is the foundational capability of any debugger. Without it, state inspection and stepping are impossible.

**Independent Test**: Can be independently tested via a new integration test simulating a DAP client connection, issuing a `setBreakpoints` request for a specific line in a defined PL/SQL procedure, executing the procedure, and verifying that a `stopped` event (reason: `breakpoint`) is emitted by the server.

**Acceptance Scenarios**:

1. **Given** a PL/SQL procedure defined in the database and a DAP client connected to the Oxibase debug port, **When** the developer sets a breakpoint on line 5 of the procedure and executes it, **Then** the procedure execution pauses before evaluating line 5, and the DAP client receives a `stopped` event.

---

### User Story 2 - Inspect Local Variables and State (Priority: P2)

As a database developer debugging a paused PL/SQL procedure, I want to inspect the current state of local variables and arguments, so that I can understand the data driving the execution logic and identify bugs.

**Why this priority**: Once execution is paused, the most immediate need is to see the data context. This relies on the core pausing capability (P1).

**Independent Test**: Can be tested via an integration test that hits a breakpoint, then issues DAP `stackTrace`, `scopes`, and `variables` requests to verify the PL/SQL environment stack frames are correctly mapped and reported to the client.

**Acceptance Scenarios**:

1. **Given** execution is paused at a breakpoint inside a PL/SQL procedure, **When** the developer inspects variables in their IDE, **Then** the IDE displays the current values of all local variables and input arguments defined in the current stack frame.

---

### User Story 3 - Step Through Execution (Priority: P3)

As a database developer debugging a PL/SQL procedure, I want to use standard debugging commands like "Step Over" and "Continue" to navigate through the code statement by statement or until the next breakpoint, so that I can trace logic errors efficiently.

**Why this priority**: Stepping provides granular control over execution flow after pausing and inspecting state.

**Independent Test**: Can be tested via an integration test that hits a breakpoint, issues a `next` (Step Over) command, and verifies execution pauses at the next statement, followed by a `continue` command that runs until completion or the next breakpoint.

**Acceptance Scenarios**:

1. **Given** execution is paused at line 5, **When** the developer issues a "Step Over" command, **Then** line 5 evaluates, and execution pauses before evaluating line 6 (or the next logical statement).
2. **Given** execution is paused, **When** the developer issues a "Continue" command, **Then** execution resumes until it finishes or hits another breakpoint.

---

### Edge Cases

- What happens when a breakpoint is set on an empty line, a comment, or a line that does not contain an executable statement? (Does it snap to the next valid line, or remain unverified?)
- How does the debugger handle unhandled exceptions raised within the PL/SQL procedure? Does it pause on exception?
- How does debugging a procedure affect concurrent transactions and MVCC? Does the paused transaction hold locks indefinitely?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST retain accurate `line_number` and source mapping metadata within the PL/SQL Abstract Syntax Tree (AST) during parsing.
- **FR-002**: The `PlSqlInterpreter::execute()` loop MUST expose an interception point (hook) prior to evaluating each statement to check for active breakpoints.
- **FR-003**: The system MUST map the internal PL/SQL `Environment` (representing scopes and stack frames) to standard DAP variable representations.
- **FR-004**: The PL/SQL debugging capabilities MUST integrate with the shared `DebugController` architecture established for other execution environments (e.g., Rhai).
- **FR-005**: The system MUST support DAP commands: `setBreakpoints`, `stackTrace`, `scopes`, `variables`, `next` (Step Over), and `continue`.

### Key Entities *(include if feature involves data)*

- **`PlSqlStatement` / AST Nodes**: The parsed representation of the code, which must now carry source location data.
- **`PlSqlInterpreter`**: The execution engine for native procedures, modified to pause and resume.
- **`Environment` (PL/SQL Stack Frame)**: The state container holding variables, mapped to DAP variable structures.
- **`DebugController`**: The central coordinator managing DAP client communication and orchestrating breakpoints across different execution engines.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can successfully connect a standard DAP client (e.g., VS Code) to an Oxibase instance and debug a native PL/SQL procedure without crashes.
- **SC-002**: Breakpoints hit accurately correspond to the intended lines in the original source code.
- **SC-003**: Variable inspection accurately reflects the types and values of PL/SQL variables at runtime.
- **SC-004**: No performance regression is observed in PL/SQL execution when a debugger is *not* attached.

## Assumptions

- A foundational DAP server infrastructure (`DebugController`) is already partially or fully implemented as part of the broader `v0.6.0` observability goals, specifically for Rhai support.
- The DAP implementation relies on standard TCP socket communication.
