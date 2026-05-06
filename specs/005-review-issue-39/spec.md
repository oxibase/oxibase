# Feature Specification: App Scaffolding and Seeding CLI Commands

**Feature Branch**: `[###-feature-name]`  
**Created**: May 06 2026
**Status**: Draft  
**Input**: User description: "Can you review issue #39 ?" (Issue #39: [Feature] App Scaffolding and Seeding CLI Commands (create-app & seed))

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create a New App Scaffold (Priority: P1)

As a developer, I want to run a single CLI command to generate a standard directory structure with boilerplate code, so that I can immediately start building my Oxibase application without having to manually create directories and initial files.

**Why this priority**: Creating the app structure is the fundamental first step in the developer workflow. Without it, the `seed` command has no standardized input to work with.

**Independent Test**: Can be fully tested by a new integration test in `tests/` that executes `oxibase create-app test-app`, verifies the `test-app` directory and its required subdirectories (`data`, `templates`, `routes`, `functions`) are created, and checks that the default files contain the expected boilerplate content.

**Acceptance Scenarios**:

1. **Given** I am in a terminal, **When** I run `oxibase create-app my-new-app`, **Then** a directory named `my-new-app` is created with the subdirectories `data/`, `templates/`, `routes/`, and `functions/`.
2. **Given** the app directory is created, **When** I inspect it, **Then** I find default files like `data/001_init.sql`, `templates/layout.html`, `routes/web.json`, and a default function script, all containing valid initial code.
3. **Given** I run `oxibase create-app`, **When** the process finishes, **Then** I see a success message instructing me how to seed the app.

---

### User Story 2 - Seed Database from App Directory (Priority: P1)

As a developer, I want to run a single CLI command to read my app's directory structure and load its schemas, templates, routes, and functions into an Oxibase database file, so that I can deterministically deploy or update my application's state.

**Why this priority**: Seeding is the core mechanism for getting the developer's declarative application definitions into the running database engine.

**Independent Test**: Can be tested by creating a mock app directory in a temporary folder, running `oxibase seed <mock-app> -d file://<temp-db>`, and then querying the `<temp-db>` to verify that the tables were created, templates inserted, and routes defined as specified in the mock files.

**Acceptance Scenarios**:

1. **Given** a valid app directory, **When** I run `oxibase seed my-app -d file:///target.db`, **Then** the database is initialized with system schemas, old app state is cleared, and new data, templates, routes, and functions are loaded within a single transaction.
2. **Given** an app directory with multiple `.sql` files in `data/`, **When** I run `seed`, **Then** the SQL files are executed in alphabetical order.
3. **Given** a corrupted or invalid file in the app directory (e.g., malformed JSON in routes), **When** I run `seed`, **Then** the seeding process fails, the database transaction is rolled back, and an informative error is displayed.

---

### Edge Cases

- What happens if `oxibase create-app` is run with a directory name that already exists?
- How does `seed` handle missing subdirectories (e.g., no `functions/` folder)?
- What happens if a SQL file in the `data/` directory contains a syntax error during seeding?
- How are very large template files handled during insertion?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST support a `create-app <name>` command that generates a predefined directory structure (`data`, `templates`, `routes`, `functions`).
- **FR-002**: The `create-app` command MUST populate the generated directories with functional boilerplate files (`001_init.sql`, `layout.html`, `index.html`, `web.json`, and a sample script).
- **FR-003**: The CLI MUST support a `seed <app_dir> --db <connection_string>` command.
- **FR-004**: The `seed` command MUST execute entirely within a single database transaction. If any step fails, the entire seed operation MUST rollback.
- **FR-005**: The `seed` command MUST create necessary system schemas (e.g., `routes`, `templates`) if they do not exist.
- **FR-006**: The `seed` command MUST clear previous application state (routes, templates, functions) before loading new ones to ensure a deterministic state.
- **FR-007**: The `seed` command MUST execute all `.sql` files found in the `data/` directory in alphabetical order.
- **FR-008**: The `seed` command MUST read all files in the `templates/` directory and insert their contents into the `templates.source` table, using the filename or relative path as the name.
- **FR-009**: The `seed` command MUST parse JSON files in the `routes/` directory and insert the definitions into the `routes.definitions` table.
- **FR-010**: The `seed` command MUST load scripts from the `functions/` directory into the appropriate system functions table.
- **FR-011**: The system MUST abort the `create-app` operation and return an error if the target directory already exists.

### Key Entities

- **App Directory**: The structured folder containing the application's declarative definition.
- **System Tables (`routes.definitions`, `templates.source`)**: The internal database tables where the app's configuration is stored for runtime use.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can generate a new app structure in under 1 second.
- **SC-002**: The `seed` command successfully loads the default boilerplate app into an empty database without errors.
- **SC-003**: An integration test verifies that seeding a previously seeded database correctly replaces the old templates and routes.
- **SC-004**: The feature adds no new `unwrap()` or `expect()` calls in the library code, passing `make lint`.

## Assumptions

- The underlying database engine already supports the required SQL statements for inserting into system tables (e.g., `templates.source`).
- The connection string format (e.g., `file:///...`) is already handled by existing connection logic.
