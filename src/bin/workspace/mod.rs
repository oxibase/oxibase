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
    println!("Installing polished Workspace app into database...");

    // Create schemas and tables if they don't exist (DDL outside transaction)
    let _ = db.execute("CREATE SCHEMA IF NOT EXISTS interface", ());
    let _ = db.execute(
        "CREATE TABLE IF NOT EXISTS interface.routes (method TEXT, path TEXT, template_name TEXT, context_query TEXT)",
        (),
    );
    let _ = db.execute(
        "CREATE TABLE IF NOT EXISTS interface.templates (name TEXT, content TEXT)",
        (),
    );

    let mut tx = db.begin().expect("Failed to start transaction");

    // Clean up existing records to make installation idempotent
    tx.execute("DELETE FROM interface.routes", ())
        .unwrap_or_default();
    tx.execute("DELETE FROM interface.templates", ())
        .unwrap_or_default();

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
    let run_modal_html = include_str!("templates/workspace_run_modal.html");
    let debugger_html = include_str!("templates/workspace_debugger.html");
    let observe_logs_html = include_str!("templates/workspace_observe_logs.html");
    let observe_traces_html = include_str!("templates/workspace_observe_traces.html");
    let pizza_demo_sql = include_str!("templates/pizza_demo.sql");

    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_layout.html', ?)",
        vec![Value::text(layout_html)],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_sidebar_data.html', ?)",
        vec![Value::text(sidebar_data_html)],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_sidebar_compute.html', ?)",
        vec![Value::text(sidebar_compute_html)],
    ).unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_sidebar_observe.html', ?)",
        vec![Value::text(sidebar_observe_html)],
    ).unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_sql_editor.html', ?)",
        vec![Value::text(editor_html)],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_sql_results.html', ?)",
        vec![Value::text(results_html)],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_table_create.html', ?)",
        vec![Value::text(table_create_html)],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_data_grid.html', ?)",
        vec![Value::text(data_grid_html)],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_trace_view.html', ?)",
        vec![Value::text(trace_view_html)],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_run_modal.html', ?)",
        vec![Value::text(run_modal_html)],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_debugger.html', ?)",
        vec![Value::text(debugger_html)],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_observe_logs.html', ?)",
        vec![Value::text(observe_logs_html)],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO interface.templates (name, content) VALUES ('workspace_observe_traces.html', ?)",
        vec![Value::text(observe_traces_html)],
    )
    .unwrap();

    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace', 'workspace_sidebar_compute.html', NULL)",
        ()
    ).unwrap();

    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/sidebar/data', 'workspace_sidebar_data.html', 'SELECT table_schema, table_name FROM information_schema.tables WHERE table_schema NOT IN (''system'', ''information_schema'', ''interface'') ORDER BY table_schema, table_name')",
        ()
    ).unwrap();

    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/sidebar/compute', 'workspace_sidebar_compute.html', NULL)",
        ()
    ).unwrap();

    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/sidebar/observe', 'workspace_sidebar_observe.html', NULL)",
        ()
    ).unwrap();

    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/sql_editor', 'workspace_sql_editor.html', 'SELECT table_schema, table_name, column_name FROM information_schema.columns ORDER BY table_schema, table_name, ordinal_position')",
        ()
    ).unwrap();

    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/meta/tables/new', 'workspace_table_create.html', NULL)",
        ()
    ).unwrap();

    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/debugger', 'workspace_debugger.html', NULL)",
        ()
    ).unwrap();
    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('POST', '/workspace/sql', 'workspace_sql_results.html', 'dummy')",
        ()
    ).unwrap();
    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/observe/logs', 'workspace_observe_logs.html', 'SELECT id, timestamp, level, target, message, json_fields, trace_id, span_id FROM system.logs WHERE (:level = ''all'' OR level = :level) ORDER BY timestamp DESC LIMIT 100')",
        ()
    ).unwrap();
    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/observe/traces', 'workspace_observe_traces.html', 'SELECT trace_id, parent_span_id, name, start_time, end_time, duration_ms, status_code FROM system.traces ORDER BY start_time DESC')",
        ()
    ).unwrap();
    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/traces/{trace_id}', 'workspace_trace_view.html', 'SELECT span_id, parent_span_id, name, span_kind, start_time, end_time, duration_ms, status_code, status_message, attributes, events FROM system.traces WHERE trace_id = :trace_id ORDER BY start_time ASC')",
        ()
    ).unwrap();
    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/run_modal', 'workspace_run_modal.html', 'SELECT parameters FROM system.procedures WHERE name = UPPER(:procedure_name) UNION ALL SELECT parameters FROM system.functions WHERE name = UPPER(:procedure_name)')",
        ()
    ).unwrap();
    tx.execute(
        "INSERT INTO interface.routes (method, path, template_name, context_query) VALUES ('GET', '/workspace/data/{schema}/{table}', 'workspace_data_grid.html', 'SELECT * FROM :schema.:table LIMIT 100')",
        ()
    ).unwrap();

    tx.commit().expect("Failed to commit transaction");

    // Run pizza demo setup script

    // We split by ';' since db.execute doesn't support multiple statements well,
    // though we have to be careful not to split inside DO blocks/PLSQL strings.
    // The easiest is just executing the full script if execute() supports it,
    // but the safer approach for Oxibase is a custom small parser or just running
    // it line by line. Given the complex triggers, executing the statements directly
    // might be tricky if we just split by ';'. Wait, I'll use a regex or split
    // carefully, or maybe Oxibase supports multiple statements.
    // Let's look at how execute works.
    let pizza_queries = parse_sql_script(pizza_demo_sql);
    for q in pizza_queries {
        if !q.trim().is_empty() {
            if let Err(e) = db.execute(&q, ()) {
                eprintln!("Failed to execute pizza demo query: {}\nError: {:?}", q, e);
            }
        }
    }

    println!("Workspace installation complete.");
}

// Simple script splitter that avoids splitting inside string literals ('...') or dollar quotes ($$...$$)
fn parse_sql_script(script: &str) -> Vec<String> {
    let mut queries = Vec::new();
    let mut current_query = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_dollar_quote = false;

    let chars: Vec<char> = script.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if in_single_quote {
            if c == '\'' {
                // Handle escaping: ''
                if i + 1 < chars.len() && chars[i + 1] == '\'' {
                    current_query.push(c);
                    current_query.push(chars[i + 1]);
                    i += 2;
                    continue;
                } else {
                    in_single_quote = false;
                }
            }
        } else if in_double_quote {
            if c == '"' {
                if i + 1 < chars.len() && chars[i + 1] == '"' {
                    current_query.push(c);
                    current_query.push(chars[i + 1]);
                    i += 2;
                    continue;
                } else {
                    in_double_quote = false;
                }
            }
        } else if in_dollar_quote {
            if c == '$' && i + 1 < chars.len() && chars[i + 1] == '$' {
                in_dollar_quote = false;
                current_query.push('$');
                current_query.push('$');
                i += 2;
                continue;
            }
        } else {
            if c == '\'' {
                in_single_quote = true;
            } else if c == '"' {
                in_double_quote = true;
            } else if c == '$' && i + 1 < chars.len() && chars[i + 1] == '$' {
                in_dollar_quote = true;
                current_query.push('$');
                current_query.push('$');
                i += 2;
                continue;
            } else if c == ';' {
                queries.push(current_query.trim().to_string());
                current_query.clear();
                i += 1;
                continue;
            } else if c == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
                // Skip comment line
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
        }

        current_query.push(c);
        i += 1;
    }

    if !current_query.trim().is_empty() {
        queries.push(current_query.trim().to_string());
    }

    queries
}

// ponytail: force template rebuild
