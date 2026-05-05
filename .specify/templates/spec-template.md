# Feature Specification: [FEATURE NAME]

**Feature Branch**: `[###-feature-name]`  
**Created**: [DATE]  
**Status**: Draft  
**Input**: User description: "$ARGUMENTS"

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

### User Story 1 - [Brief Title] (Priority: P1)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently - e.g., "Can be fully tested by a new integration test in `tests/` verifying `make test` output"]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [SQL command or API call], **Then** [expected result/dataset/error]
2. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 2 - [Brief Title] (Priority: P2)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

- What happens when [invalid SQL syntax is provided]?
- How does system handle [NULL values or type mismatches]?
- How does MVCC isolate this operation during concurrent transactions?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Engine MUST [specific capability, e.g., "parse and execute window functions"]
- **FR-002**: Storage MUST [specific capability, e.g., "maintain ACID guarantees during execution"]  
- **FR-003**: System MUST NOT [e.g., "allocate new Vecs inside the hot loop"]

*Example of marking unclear requirements:*

- **FR-004**: System MUST support [NEEDS CLARIFICATION: exact data types required - VARCHAR, INT64, FLOAT?]

### Key Entities *(include if feature involves data)*

- **[SQL AST Node]**: [What it represents in the parser]
- **[Logical Plan Node]**: [What it represents in the optimizer]
- **[Physical Execution Node]**: [What it represents in the executor]

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: [Measurable metric, e.g., "Passes all new and existing `make test` suites"]
- **SC-002**: [Measurable metric, e.g., "Performance regression is < 5% as measured by benchmark suite"]
- **SC-003**: [Code quality metric, e.g., "Passes `make lint` without warnings and no new `unwrap()` calls introduced"]

## Assumptions

- [Assumption about SQL standard compliance]
- [Assumption about storage engine behavior]
