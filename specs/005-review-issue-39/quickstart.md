# Quickstart: App Scaffolding and Seeding

This guide shows you how to use the new `create-app` and `seed` commands to build Oxibase applications.

## 1. Scaffold your App

Use `create-app` to generate a standard directory structure:

```bash
oxibase create-app my-blog
```

This generates:
```text
my-blog/
├── data/
│   └── 001_init.sql
├── templates/
│   ├── layout.html
│   └── index.html
├── routes/
│   └── web.json
└── functions/
    └── hello.rhai
```

*If `my-blog/` already exists, the command will abort to prevent overwriting your files.*

## 2. Develop your App

Edit the files inside `my-blog/`:
- Add new tables and data in `data/*.sql`.
- Write your HTML layouts in `templates/*.html`.
- Define your API and Page routes in `routes/*.json`.
- Add custom logic in `functions/*.rhai`.

## 3. Seed your Database

When you are ready to load your application into Oxibase, use the `seed` command:

```bash
oxibase seed my-blog -d file:///prod.db
```

This will run everything inside a single, safe transaction. If any file has an error, the entire operation is rolled back, protecting your database state.

## 4. Run the Server

Start serving your application:

```bash
oxibase serve -d file:///prod.db
```