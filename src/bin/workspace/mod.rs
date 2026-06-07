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
    let _ = tx.execute("CREATE SCHEMA IF NOT EXISTS interface", ());
    let _ = tx.execute(
        "CREATE TABLE IF NOT EXISTS interface.routes (method TEXT, path TEXT, template_name TEXT, context_query TEXT)",
        (),
    );
    let _ = tx.execute(
        "CREATE TABLE IF NOT EXISTS interface.templates (name TEXT, content TEXT)",
        (),
    );
    let _ = tx.execute(
        "CREATE TABLE IF NOT EXISTS interface.templates (name TEXT, content TEXT)",
        (),
    );

    // Clean up existing records to make installation idempotent
    let _ = tx.execute("DELETE FROM interface.routes", ());
    let _ = tx.execute("DELETE FROM interface.templates", ());

    // Load templates via include_str!
    let layout_html = include_str!("templates/workspace_layout.html");
    let sidebar_data_html = include_str!("templates/workspace_sidebar_data.html");
    let sidebar_compute_html = include_str!("templates/workspace_sidebar_compute.html");
    let sidebar_observe_html = include_str!("templates/workspace_sidebar_observe.html");
    let editor_html = include_str!("templates/workspace_sql_editor.html");
    let results_html = include_str!("templates/workspace_sql_results.html");
    let table_create_html = include_str!("templates/workspace_table_create.html");
    let data_grid_html = include_str!("templates/workspace_data_grid.html");
    let trace_view_html = include_str!("templates/workspace_trace_view.html");

    let _ = tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_layout.html', ?)",
        vec![Value::text(layout_html)],
    );
    let _ = tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_sidebar_data.html', ?)",
        vec![Value::text(sidebar_data_html)],
    );
    let _ = tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_sidebar_compute.html', ?)",
        vec![Value::text(sidebar_compute_html)],
    );
    let _ = tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_sidebar_observe.html', ?)",
        vec![Value::text(sidebar_observe_html)],
    );
    let _ = tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_sql_editor.html', ?)",
        vec![Value::text(editor_html)],
    );
    let _ = tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_sql_results.html', ?)",
        vec![Value::text(results_html)],
    );
    let _ = tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_table_create.html', ?)",
        vec![Value::text(table_create_html)],
    );
    let _ = tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_data_grid.html', ?)",
        vec![Value::text(data_grid_html)],
    );
    let _ = tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_trace_view.html', ?)",
        vec![Value::text(trace_view_html)],
    );

    let _ = tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace', 'workspace_layout.html', NULL)",
        ()
    );

    let _ = tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/sidebar/data', 'workspace_sidebar_data.html', 'SELECT table_schema, table_name FROM information_schema.tables ORDER BY table_schema, table_name')",
        ()
    );

    let _ = tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/sidebar/compute', 'workspace_sidebar_compute.html', NULL)",
        ()
    );

    let _ = tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/sidebar/observe', 'workspace_sidebar_observe.html', NULL)",
        ()
    );

    let _ = tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/sql_editor', 'workspace_sql_editor.html', NULL)",
        ()
    );

    let _ = tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/meta/tables/new', 'workspace_table_create.html', NULL)",
        ()
    );

    tx.commit().expect("Failed to commit transaction");
    println!("Workspace installation complete.");
}
