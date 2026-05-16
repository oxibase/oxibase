# Feature Specification: context-autocomplete

**Feature Branch**: `020-context-autocomplete`
**Created**: May 16, 2026
**Status**: Draft
**Input**: User description: "can you add a context-aware autocomplete ?"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - SQL Keyword Autocomplete (Priority: P1)

Users typing SQL queries in the CLI can hit Tab to complete standard SQL keywords (SELECT, INSERT, CREATE, etc.) or CLI commands (help, exit), speeding up their workflow.

**Why this priority**: Basic keyword autocomplete is the foundation of any interactive SQL shell, providing immediate value with minimal context dependency.

**Independent Test**: Can be tested independently by mocking the CLI input and verifying the `rustyline::completion::Completer` returns the correct keyword suggestions for partial inputs.

**Acceptance Scenarios**:

1. **Given** an empty prompt, **When** the user types `SEL` and hits Tab, **Then** the CLI suggests `SELECT`.
2. **Given** a prompt with `CREATE `, **When** the user types `TAB` and hits Tab, **Then** the CLI suggests `TABLE`.
3. **Given** a prompt, **When** the user types `he` and hits Tab, **Then** the CLI suggests `help`.

---

### User Story 2 - Context-Aware Schema Autocomplete (Priority: P2)

Users typing SQL queries in the CLI can hit Tab to complete database schema objects, such as table names after `FROM` or `UPDATE`, based on the actual current state of the database.

**Why this priority**: Context-awareness provides massive value by saving users from having to remember exact table or column names, but it requires deeper integration with the database state.

**Independent Test**: Can be tested by creating a dummy database with known tables, invoking the completer with a partial query string like `SELECT * FROM m`, and verifying it returns matching table names.

**Acceptance Scenarios**:

1. **Given** a database with a table named `users`, **When** the user types `SELECT * FROM us` and hits Tab, **Then** the CLI suggests `users`.
2. **Given** a database with tables `orders` and `order_items`, **When** the user types `SELECT * FROM ord` and hits Tab, **Then** the CLI suggests both `orders` and `order_items`.

### Edge Cases

- What happens when the user hits Tab on a completely empty line? (Should probably show nothing or top-level commands).
- How does the system handle table names with spaces or special characters?
- How does the autocomplete perform on a database with thousands of tables? (Need to ensure it doesn't hang the UI).
- What if there are multiple completions for a partial string?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST implement a completion helper that hooks into the readline interface.
- **FR-002**: The autocomplete MUST provide suggestions for standard SQL keywords and CLI-specific commands.
- **FR-003**: The autocomplete MUST connect to the active database instance to dynamically fetch and cache (or quickly query) schema object names (like tables) for context-aware suggestions.
- **FR-004**: The system MUST return autocomplete suggestions in under 100ms to maintain a responsive interactive feel.

### Key Entities

- **[SQL Helper]**: The component bridging the readline interface with the autocomplete logic.
- **[Completer]**: The logic evaluating the current cursor position and line text to generate suggestions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully autocomplete at least 20 common SQL keywords and CLI commands.
- **SC-002**: Users can autocomplete existing table names in the connected database.
- **SC-003**: Autocomplete suggestion generation takes less than 50ms on average, ensuring no noticeable lag for the user.
- **SC-004**: Implementation passes all linters (`make lint`) and introduces no new panics or unwrap calls.

## Assumptions

- We assume the user wants standard SQL keywords as the base of the autocomplete.
- We assume schema object completion will focus primarily on table names for this initial iteration, and column names might be deferred if too complex.
