# Specification: Deprecate Boa JavaScript Engine

## Feature Description
Deprecate and remove the JavaScript (Boa) scripting engine from the database. This involves removing the dependency, feature flags, related backend implementation code, updating tests, and removing references from the documentation.

## User Scenarios & Testing

### Scenario 1: Building the Database
**Given** a developer is building the database from source
**When** they run `cargo build` or `cargo build --all-features`
**Then** the build succeeds without attempting to compile the `boa_engine` dependency or any JavaScript backend code.

### Scenario 2: Documentation Review
**Given** a user is reading the documentation (e.g., `README.md`)
**When** they look for supported scripting languages
**Then** JavaScript and `boa_engine` are no longer listed as supported features or options.

## Functional Requirements

1. **Dependency Removal**: The `boa_engine` dependency must be completely removed from `Cargo.toml`.
2. **Feature Flag Removal**: The `js` feature flag must be removed from `Cargo.toml`.
3. **Backend Code Removal**: The JavaScript backend implementation (`src/functions/backends/boa.rs` and related stubs) must be deleted.
4. **Registry Update**: The backend registry must no longer attempt to register the JavaScript backend.
5. **Test Cleanup**: All tests specifically targeting the JavaScript backend must be removed or rewritten if they test multi-language infrastructure.
6. **Documentation Update**: `README.md` and any other user-facing documentation must be updated to remove references to the `js` feature and the JavaScript/Boa backend.
7. **CI Update**: Continuous Integration pipelines must no longer pass the `--features js` flag during test or build steps.

## Non-Functional Requirements

1. **Maintainability**: The removal should leave the scripting backend infrastructure clean and intact for remaining languages (Rhai, Python).
2. **Build Times**: Removing the `boa_engine` dependency should marginally improve compilation times and reduce the final binary size.

## Success Criteria

- `cargo check --all-features` runs successfully.
- `cargo test --all-features` runs successfully with no skipped JavaScript tests (because they no longer exist).
- A text search for `boa_engine` in the codebase yields 0 results.
- A text search for `feature = "js"` in the codebase yields 0 results.

## Assumptions & Out of Scope

- **Assumptions**: We assume that existing users relying on the JavaScript backend will migrate to Rhai or Python.
- **Out of Scope**: Providing a migration tool or script for users to convert their existing JS stored procedures to Rhai or Python is out of scope. We are strictly removing the backend support.