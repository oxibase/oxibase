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

//! Askama templates for the forum application

use askama::Template;

use crate::web::models::*;

// Template-compatible structs (askama requires Display for all fields)

use super::models::OptionalString;

#[derive(Debug, Clone)]
pub struct CategoryTemplate {
    pub id: i64,
    pub name: String,
    pub description: OptionalString,
    pub sort_order: i64,
}

#[derive(Debug, Clone)]
pub struct CategoryWithStatsTemplate {
    pub category: CategoryTemplate,
    pub thread_count: i64,
    pub post_count: i64,
    pub last_thread: Option<ForumThread>,
}

#[derive(Template)]
#[template(path = "forum/index.html")]
pub struct ForumIndex {
    pub categories: Vec<CategoryWithStatsTemplate>,
}

#[derive(Template)]
#[template(path = "forum/category.html")]
pub struct CategoryPage {
    pub category: CategoryTemplate,
    pub threads: Vec<ThreadWithMeta>,
}

#[derive(Template)]
#[template(path = "forum/thread.html")]
pub struct ThreadPage {
    pub thread: ForumThread,
    pub posts: Vec<PostWithAuthor>,
}

#[derive(Template)]
#[template(path = "forum/threads_fragment.html")]
pub struct ThreadsFragment {
    pub threads: Vec<ThreadWithMeta>,
}

#[derive(Template)]
#[template(path = "forum/posts_fragment.html")]
pub struct PostsFragment {
    pub posts: Vec<PostWithAuthor>,
}