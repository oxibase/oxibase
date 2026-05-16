# Quickstart: Public Schema Fallback

## Overview

This feature ensures that whenever a database object (such as a table, view, sequence, procedure, function, or trigger) is created without explicitly specifying a schema, it defaults to the `public` schema.

## Developer Guide

1. **Review Context**: Familiarize yourself with `src/executor/ddl.rs` and `src/storage/mvcc/engine.rs`.
2. **Implement Fallback**: Update the object creation methods in `ddl.rs` to extract the schema via `ctx.current_schema().unwrap_or("public")` instead of letting it default to `None`/`NULL`.
3. **Refactor Storage Maps**: In `MVCCEngine`, modify `views` and `sequences` to use a nested `FxHashMap` mapped by schema name.
4. **Update Tests**: Verify existing tests pass (`make test`). You may need to add new tests to explicitly test objects being dropped/created from the `public` schema.
