// Copyright 2025 Stoolap Contributors
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

//! PostgreSQL wire protocol implementation for Oxibase

use std::sync::Arc;

use async_trait::async_trait;
use pgwire::api::query::SimpleQueryHandler;
use pgwire::api::results::Response;
use pgwire::api::PgWireServerHandlers;
use pgwire::error::{ErrorInfo, PgWireResult};

use crate::api::Database;

/// Backend factory for creating Oxibase backends
pub struct OxiBaseBackendFactory {
    db: Database,
}

impl OxiBaseBackendFactory {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl PgWireServerHandlers for OxiBaseBackendFactory {
    fn simple_query_handler(&self) -> Arc<impl SimpleQueryHandler> {
        Arc::new(OxiBaseBackend::new(self.db.clone()))
    }
}

/// Oxibase backend implementation
pub struct OxiBaseBackend {
    db: Database,
}

impl OxiBaseBackend {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SimpleQueryHandler for OxiBaseBackend {
    async fn do_query<C>(
        &self,
        _client: &mut C,
        query: &str,
    ) -> PgWireResult<Vec<Response>>
    where
        C: pgwire::api::ClientInfo + Unpin + Send + Sync,
    {
        // For now, just return an error for any query
        // TODO: Implement proper query handling
        Ok(vec![Response::Error(Box::new(ErrorInfo::new(
            "ERROR".to_owned(),
            "XX000".to_owned(),
            format!("Query not yet implemented: {}", query),
        )))])
    }
}

