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

//! Web server module for Oxibase forum application
//!
//! This module provides web server functionality to serve a forum
//! application directly from an Oxibase database.

pub mod handlers;
pub mod models;
pub mod server;
pub mod templates;

pub use server::{start_server, init_forum_schema};