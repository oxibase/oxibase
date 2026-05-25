// Copyright 2025 Stoolap Contributors
// Copyright 2025 Oxibase Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use oxibase::api::Database;
use oxibase::Value;

pub fn install(db: &Database) {
    println!("Installing Workspace app into database...");
    let mut tx = db.begin().expect("Failed to start transaction");

    // Create schemas and tables if they don't exist
    let _ = tx.execute("CREATE SCHEMA IF NOT EXISTS routes", ());
    let _ = tx.execute("CREATE SCHEMA IF NOT EXISTS templates", ());
    let _ = tx.execute(
        "CREATE TABLE IF NOT EXISTS routes.definitions (method TEXT, path TEXT, template_name TEXT, context_query TEXT)",
        ()
    );
    let _ = tx.execute(
        "CREATE TABLE IF NOT EXISTS templates.source (name TEXT, content TEXT)",
        (),
    );

    // Clean up existing records to make installation idempotent
    let _ = tx.execute("DELETE FROM routes.definitions", ());
    let _ = tx.execute("DELETE FROM templates.source", ());

    // Load templates via include_str!
    let layout_html = include_str!("templates/workspace_layout.html");
    let sidebar_html = include_str!("templates/workspace_sidebar.html");
    let editor_html = include_str!("templates/workspace_sql_editor.html");
    let results_html = include_str!("templates/workspace_sql_results.html");
    let table_create_html = include_str!("templates/workspace_table_create.html");
    let data_grid_html = include_str!("templates/workspace_data_grid.html");

    let _ = tx.execute(
        "INSERT INTO templates.source (name, content) VALUES ('workspace_layout.html', ?)",
        vec![Value::text(layout_html)],
    );
    let _ = tx.execute(
        "INSERT INTO templates.source (name, content) VALUES ('workspace_sidebar.html', ?)",
        vec![Value::text(sidebar_html)],
    );
    let _ = tx.execute(
        "INSERT INTO templates.source (name, content) VALUES ('workspace_sql_editor.html', ?)",
        vec![Value::text(editor_html)],
    );
    let _ = tx.execute(
        "INSERT INTO templates.source (name, content) VALUES ('workspace_sql_results.html', ?)",
        vec![Value::text(results_html)],
    );
    let _ = tx.execute(
        "INSERT INTO templates.source (name, content) VALUES ('workspace_table_create.html', ?)",
        vec![Value::text(table_create_html)],
    );
    let _ = tx.execute(
        "INSERT INTO templates.source (name, content) VALUES ('workspace_data_grid.html', ?)",
        vec![Value::text(data_grid_html)],
    );

    let _ = tx.execute(
        "INSERT INTO routes.definitions (method, path, template_name, context_query) VALUES ('GET', '/workspace', 'workspace_layout.html', NULL)",
        ()
    );

    let _ = tx.execute(
        "INSERT INTO routes.definitions (method, path, template_name, context_query) VALUES ('GET', '/workspace/sidebar', 'workspace_sidebar.html', 'SELECT table_schema, table_name FROM information_schema.tables ORDER BY table_schema, table_name')",
        ()
    );

    let _ = tx.execute(
        "INSERT INTO routes.definitions (method, path, template_name, context_query) VALUES ('GET', '/workspace/sql_editor', 'workspace_sql_editor.html', NULL)",
        ()
    );

    let _ = tx.execute(
        "INSERT INTO routes.definitions (method, path, template_name, context_query) VALUES ('GET', '/workspace/meta/tables/new', 'workspace_table_create.html', NULL)",
        ()
    );

    tx.commit().expect("Failed to commit transaction");
    println!("Workspace installation complete.");
}
