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

//! Forum data models for the web server

use oxibase::{FromRow, Value};
use chrono::{DateTime, Utc};
use std::fmt;

// Custom Display implementation for Option<String> to work with askama
impl fmt::Display for crate::web::models::OptionalString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionalString::Some(s) => write!(f, "{}", s),
            OptionalString::None => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OptionalString {
    Some(String),
    None,
}

impl From<Option<String>> for OptionalString {
    fn from(opt: Option<String>) -> Self {
        match opt {
            Some(s) => OptionalString::Some(s),
            None => OptionalString::None,
        }
    }
}

/// Forum user
#[derive(Debug, Clone, FromRow)]
pub struct ForumUser {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// Forum category
#[derive(Debug, Clone, FromRow)]
pub struct ForumCategory {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i64,
}

/// Forum thread
#[derive(Debug, Clone, FromRow)]
pub struct ForumThread {
    pub id: i64,
    pub category_id: i64,
    pub user_id: i64,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_pinned: bool,
    pub is_locked: bool,
}

/// Forum post
#[derive(Debug, Clone, FromRow)]
pub struct ForumPost {
    pub id: i64,
    pub thread_id: i64,
    pub user_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Thread with additional data
#[derive(Debug, Clone)]
pub struct ThreadWithMeta {
    pub thread: ForumThread,
    pub author: ForumUser,
    pub category: ForumCategory,
    pub post_count: i64,
    pub last_post: Option<ForumPost>,
}

/// Post with author information
#[derive(Debug, Clone)]
pub struct PostWithAuthor {
    pub post: ForumPost,
    pub author: ForumUser,
}

/// Category with thread count
#[derive(Debug, Clone)]
pub struct CategoryWithStats {
    pub category: ForumCategory,
    pub thread_count: i64,
    pub post_count: i64,
    pub last_thread: Option<ForumThread>,
}