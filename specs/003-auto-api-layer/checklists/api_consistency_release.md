# API Consistency Release Checklist: Auto-API Layer

**Purpose**: Formal release gate checklist evaluating the quality, consistency, and completeness of the API requirements.
**Created**: $(date +%Y-%m-%d)
**Target**: `spec.md`

## 1. Requirement Consistency (API Contract)
- [ ] CHK001 Are response payload formats (e.g., returning `{ "rows_affected": X }` vs arrays) consistent across all mutation endpoints (POST, PATCH, DELETE)? [Consistency, Spec §FR5-FR7]
- [ ] CHK002 Do the requirements explicitly state the standard HTTP response codes for successful mutations (201 Created vs 200 OK)? [Clarity, Spec §Scenario 4-7]
- [ ] CHK003 Are error response payloads explicitly defined (e.g., standard `{"error": "message"}` format) for consistent error handling? [Consistency, Gap]

## 2. Requirement Completeness (Query Parameters)
- [ ] CHK004 Are the default values for `limit` and `offset` explicitly defined if omitted by the client? [Completeness, Spec §FR4]
- [ ] CHK005 Is the behavior explicitly defined if a user requests multiple conflicting `order` parameters (e.g., `?order=id.desc,id.asc`)? [Edge Case, Gap]
- [ ] CHK006 Is the fallback ordering behavior explicitly defined if no `order` parameter is provided? [Completeness, Gap]
- [ ] CHK007 Is the behavior explicitly defined for `PATCH` or `DELETE` requests that omit the mandatory `eq.` filter? [Exception Flow, Gap]
- [ ] CHK008 Are multiple concurrent filters supported (e.g., `?id=eq.1&name=eq.Alice`), and if so, is the logical operator (AND/OR) explicitly defined? [Clarity, Spec §FR6]

## 3. Requirement Clarity & Measurability
- [ ] CHK009 Is "clean, standard JSON arrays" defined with exact JSON schema mapping rules for edge-case data types (e.g., Timestamps, Nulls)? [Measurability, Spec §FR9]
- [ ] CHK010 Are limits defined for maximum pagination bounds to prevent DoS via massive `limit` values? [Non-Functional, Gap]

## 4. Scenario Coverage & Exceptions
- [ ] CHK011 Are requirements defined for what happens when a client attempts to insert or update with invalid JSON structures? [Exception Flow, Scenario 4/6]
- [ ] CHK012 Are requirements defined for how the API responds when a constraint violation (e.g., Unique Key error) occurs during an INSERT/UPDATE? [Exception Flow, Gap]
- [ ] CHK013 Is the API behavior specified for empty tables (e.g., returning an empty JSON array vs 404)? [Coverage, Scenario 2]
