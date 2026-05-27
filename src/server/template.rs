// Copyright 2025 Oxibase Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::api::Database;
use minijinja::{Environment, Error as JinjaError, ErrorKind};
use std::sync::Arc;

/// A custom loader for minijinja to fetch template source from `interface.templates`
pub fn db_template_loader(name: &str, db: Arc<Database>) -> Result<Option<String>, JinjaError> {
    let query = "SELECT content FROM interface.templates WHERE name = ?";

    // Convert Value strings properly to prevent format errors
    let rows_result = db
        .query(query, vec![crate::Value::text(name)])
        .map_err(|e| {
            JinjaError::new(
                ErrorKind::TemplateNotFound,
                format!("Database error fetching template: {}", e),
            )
        })?;

    let mut rows = vec![];
    for r in rows_result.flatten() {
        rows.push(r);
    }

    if rows.is_empty() {
        return Ok(None);
    }

    let content_val = rows[0]
        .get_value(0)
        .cloned()
        .unwrap_or(crate::Value::null_unknown());

    if let crate::Value::Text(s) = content_val {
        Ok(Some(s.to_string()))
    } else {
        Ok(None)
    }
}

/// Creates a new minijinja Environment hooked up to the database templates
pub fn create_env(db: Arc<Database>) -> Environment<'static> {
    let mut env = Environment::new();
    let db_clone = db.clone();

    env.set_loader(move |name| db_template_loader(name, db_clone.clone()));

    env
}
