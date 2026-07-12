# Contracts: Workspace DOM Elements & API Routes

This document defines the interface and DOM selector contract that the Puppeteer test suite relies upon to validate the interactive GUI dashboards.

## 1. Route Navigation Contract

The Workspace relies on the following standard routes:

- **SQL Editor**: `http://localhost:8080/workspace/sql_editor`
- **Logs Explorer**: `http://localhost:8080/workspace/observe/logs`
- **Traces Explorer**: `http://localhost:8080/workspace/observe/traces`
- **Trace Details Gantt View**: `http://localhost:8080/workspace/traces/{trace_id}`

## 2. DOM Selector Contract

To guarantee that frontend refinements do not break automated tests, the Workspace HTML templates must preserve the following key ID and class definitions:

| Element Description | DOM Selector Anchor | Expected Event/Interactions |
| :--- | :--- | :--- |
| SQL Query Textarea | `textarea[name="query"]` or `#editor-textarea` | Typing query string. |
| SQL Execute Button | `button[type="submit"]` or `.btn-primary` | Triggers submit and AJAX render. |
| Query Results Container | `#query-results` or `.table-container` | Houses resulting rows. |
| Log Severity ERROR Count | `#hist-error` | Displays formatted integer string. |
| Log Severity WARN Count | `#hist-warn` | Displays formatted integer string. |
| Log Severity INFO Count | `#hist-info` | Displays formatted integer string. |
| Log Severity DEBUG Count | `#hist-debug` | Displays formatted integer string. |
| Gantt Tree Timeline Root | `#tree-container` | Contains recursively rendered timeline divs. |
| Gantt Timber bar duration | `.gantt-timing-bar` | Position and width styled in percentage values. |
