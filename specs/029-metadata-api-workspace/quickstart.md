# Workspace Quickstart Guide

## Overview
The Workspace app is a built-in database management GUI for Oxibase. It uses an Unpoly + DaisyUI (via TailwindCSS) frontend served directly from the database's internal template engine using Minijinja.

## 1. Installation
The Workspace app is not active by default. It must be explicitly installed into the database's internal routing schema.

Run the following CLI command to install the workspace routes and templates:
```bash
cargo run --bin oxibase --features cli -- install-workspace
```

## 2. Accessing the Workspace
Once installed and the Oxibase server is running, navigate your browser to:
`http://localhost:8080/workspace`

## 3. Developing the Workspace
The UI is composed using:
- **Unpoly** (`unpoly.js`): Handles AJAX partial updates and modals without writing custom JS. Use attributes like `up-target` and `up-layer`.
- **DaisyUI** (`daisyui@5`): A TailwindCSS component library for UI elements. Included via CDN:
  ```html
  <link href="https://cdn.jsdelivr.net/npm/daisyui@5" rel="stylesheet" type="text/css" />
  <script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>
  ```
- **Minijinja** (Backend): Server-side templates that render HTML fragments requested by Unpoly.

## 4. Key Workflows
- **Schema Explorer**: Use the sidebar to browse schemas, tables, and views. The UI fetches this data dynamically using the `/api/meta/*` endpoints.
- **Query Editor**: Navigate to the SQL tab to write raw queries. Results are executed via `POST /api/sql` and rendered into a DaisyUI table.
- **Data Management**: Click a table to view its rows, which interact with the `/api/data/{table}` endpoints.