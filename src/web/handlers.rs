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

//! Web handlers for the forum application

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use std::sync::Arc;
use askama::Template;
use oxibase::api::Database;

use crate::web::models::*;
use crate::web::templates::*;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}

/// Forum home page - list all categories
pub async fn forum_index(State(state): State<AppState>) -> impl IntoResponse {
    let categories = match get_categories_with_stats(&state.db) {
        Ok(cats) => cats,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Database error</h1>")).into_response(),
    };

    // Convert categories for templates
    let template_categories: Vec<CategoryWithStatsTemplate> = categories.into_iter().map(|cat| {
        CategoryWithStatsTemplate {
            category: CategoryTemplate {
                id: cat.category.id,
                name: cat.category.name,
                description: cat.category.description.into(),
                sort_order: cat.category.sort_order,
            },
            thread_count: cat.thread_count,
            post_count: cat.post_count,
            last_thread: cat.last_thread,
        }
    }).collect();

    let template = ForumIndex { categories: template_categories };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Template error</h1>")).into_response(),
    }
}

/// Category page - list threads in a category
pub async fn category_page(
    State(state): State<AppState>,
    Path(category_id): Path<i64>,
) -> impl IntoResponse {
    let category_row = match state.db.query_one(
        "SELECT * FROM forum_categories WHERE id = ?",
        (category_id,),
    ) {
        Ok(row) => row,
        Err(_) => return (StatusCode::NOT_FOUND, Html("<h1>Category not found</h1>")).into_response(),
    };

    let threads = match get_threads_in_category(&state.db, category_id) {
        Ok(threads) => threads,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Database error</h1>")).into_response(),
    };

    let category_template = CategoryTemplate {
        id: category_row.id,
        name: category_row.name,
        description: category_row.description.into(),
        sort_order: category_row.sort_order,
    };

    let template = CategoryPage { category: category_template, threads };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Template error</h1>")).into_response(),
    }
}

/// Thread page - show thread with all posts
pub async fn thread_page(
    State(state): State<AppState>,
    Path(thread_id): Path<i64>,
) -> impl IntoResponse {
    let thread = match state.db.query_one(
        "SELECT * FROM forum_threads WHERE id = ?",
        (thread_id,),
    ) {
        Ok(row) => row,
        Err(_) => return (StatusCode::NOT_FOUND, Html("<h1>Thread not found</h1>")).into_response(),
    };

    let posts = match get_posts_in_thread(&state.db, thread_id) {
        Ok(posts) => posts,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Database error</h1>.to_string())).into_response(),
    };

    let template = ThreadPage { thread, posts };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Html("<div>Template error</div>")).into_response(),
    }
}

/// Create new thread (HTMX endpoint)
pub async fn create_thread(
    State(state): State<AppState>,
    Path(category_id): Path<i64>,
    Form(form): Form<CreateThreadForm>,
) -> impl IntoResponse {
    // For now, use user_id = 1 (admin)
    let user_id = 1;

    if let Err(_) = state.db.execute(
        "INSERT INTO forum_threads (category_id, user_id, title) VALUES (?, ?, ?)",
        (category_id, user_id, form.title),
    ) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Html("<div>Error creating thread</div>"));
    }

    // Return HTMX response to refresh the thread list
    Html("<div>Thread created</div>".to_string())
}

/// Create new post (HTMX endpoint)
pub async fn create_post(
    State(state): State<AppState>,
    Path(thread_id): Path<i64>,
    Form(form): Form<CreatePostForm>,
) -> impl IntoResponse {
    // For now, use user_id = 1 (admin)
    let user_id = 1;

    if let Err(_) = state.db.execute(
        "INSERT INTO forum_posts (thread_id, user_id, content) VALUES (?, ?, ?)",
        (thread_id, user_id, form.content),
    ) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Html("<div>Error creating post</div>")).into_response();
    }

    // Return HTMX response to refresh the posts
    Html("<div>Post created</div>.to_string())
}

/// Get threads in category (HTMX fragment)
pub async fn get_threads_fragment(
    State(state): State<AppState>,
    Path(category_id): Path<i64>,
) -> impl IntoResponse {
    let threads = match get_threads_in_category(&state.db, category_id) {
        Ok(threads) => threads,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<div>Error loading threads</div>".to_string())).into_response(),
    };

    let template = ThreadsFragment { threads };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Html("<div>Template error</div>")).into_response(),
    }
}
}

/// Get posts in thread (HTMX fragment)
pub async fn get_posts_fragment(
    State(state): State<AppState>,
    Path(thread_id): Path<i64>,
) -> impl IntoResponse {
    let posts = match get_posts_in_thread(&state.db, thread_id) {
        Ok(posts) => posts,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<div>Error loading posts</div>.to_string())).into_response(),
    };

    let template = PostsFragment { posts };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Html("<div>Template error</div>")).into_response(),
    }
}
}

/// Helper functions for database queries

fn get_categories_with_stats(_db: &Database) -> Result<Vec<CategoryWithStats>, Box<dyn std::error::Error>> {
    // TODO: implement SQL query
    Ok(Vec::new())
}

    let mut categories = Vec::new();
    for row in rows {
        let row = row?;
        let category: ForumCategory = row.get(0)?;
        let thread_count: i64 = row.get(1)?;
        let post_count: i64 = row.get(2)?;
        categories.push(CategoryWithStats {
            category,
            thread_count,
            post_count,
            last_thread: None, // TODO: implement
        });
    }

    Ok(categories)
}

fn get_threads_in_category(_db: &Database, _category_id: i64) -> Result<Vec<ThreadWithMeta>, Box<dyn std::error::Error>> {
    // TODO: implement SQL query - having issues with Rust parser interpreting SQL as Rust syntax
    Ok(Vec::new())
}

    let mut threads = Vec::new();
    for row in rows {
        let row = row?;
        let thread: ForumThread = row.get(0)?;
        let author_username: String = row.get(1)?;
        let post_count: i64 = row.get(2)?;

        let author = ForumUser {
            id: thread.user_id,
            username: author_username,
            email: None,
            password_hash: String::new(),
            created_at: thread.created_at,
            last_login: None,
        };

        let category = ForumCategory {
            id: thread.category_id,
            name: String::new(), // TODO: get from query
            description: None,
            sort_order: 0,
        };

        threads.push(ThreadWithMeta {
            thread,
            author,
            category,
            post_count,
            last_post: None, // TODO: implement
        });
    }

    Ok(threads)
}

fn get_posts_in_thread(_db: &Database, _thread_id: i64) -> Result<Vec<PostWithAuthor>, Box<dyn std::error::Error>> {
    // TODO: implement
    Ok(Vec::new())
}

/// Form data structures

#[derive(serde::Deserialize)]
pub struct CreateThreadForm {
    pub title: String,
}

#[derive(serde::Deserialize)]
pub struct CreatePostForm {
    pub content: String,
}