# Feature Specification: rhai-timestamp

**Feature Branch**: `045-rhai-timestamp`
**Created**: 2026-06-18
**Status**: Draft
**Input**: User description: "can you implement timestamp in rhai ? webfetch: https://rhai.rs/book/language/values-and-types.html"

## Clarifications

### Session 2026-06-18
- Q: How should database timestamps (`Value::Timestamp`) be passed to and returned from Rhai scripts? → A: Map as a custom Rhai DateTime object with helper methods (Option C).



## User Scenarios & Testing *(mandatory)*

### User Story 1 - Rhai Script with Time (Priority: P1)

Users can measure execution time or block execution using standard time concepts (`timestamp()`, `elapsed()`, and `sleep()`) inside their Rhai scripts.

**Why this priority**: Core functionality that users expect when scripting behavior that depends on time duration or needs profiling.

**Independent Test**: Can be independently tested via `cargo nextest` or `make test` with test cases directly executing small Rhai scripts containing these time functions.

**Acceptance Scenarios**:

1. **Given** a Rhai script environment, **When** executing `let t = timestamp(); sleep(1.0); t.elapsed > 0.0`, **Then** the script should successfully execute and return `true`.
2. **Given** a Rhai script environment, **When** calculating time difference `timestamp() - timestamp()`, **Then** the script should yield a numeric value representing seconds.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST expose Rhai's native `timestamp()` function.
- **FR-002**: System MUST expose Rhai's native time-related methods on timestamps (e.g., `elapsed`, `+`, `-`).
- **FR-003**: System MUST expose Rhai's `sleep` function for script execution delays.
- **FR-004**: System MUST map Oxibase `Value::Timestamp(DateTime<Utc>)` to a custom Rhai DateTime object during argument passing.
- **FR-005**: System MUST support returning custom Rhai DateTime objects and mapping them back to Oxibase `Value::Timestamp`.


## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Rhai scripts using `timestamp()` parse and execute without errors.
- **SC-002**: `make test` runs without any regressions.
- **SC-003**: Integration of the `BasicTimePackage` (or equivalent standard time functionality) into the Rhai engine setup without introducing panic conditions.

## Assumptions

- Rust's `std::time::Instant` and associated Rhai time packages (`BasicTimePackage`, `LanguageCorePackage`) are supported by our current Rhai version and build target.
- The `rhai` crate in the project is compiled without the `no_time` feature.
