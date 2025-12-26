-- Forum Schema for Oxibase Web Server
-- This SQL creates the tables needed for the forum application

-- Users table
CREATE TABLE IF NOT EXISTS forum_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP
);

-- Categories table
CREATE TABLE IF NOT EXISTS forum_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    sort_order INTEGER DEFAULT 0
);

-- Threads table
CREATE TABLE IF NOT EXISTS forum_threads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_pinned BOOLEAN DEFAULT FALSE,
    is_locked BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (category_id) REFERENCES forum_categories(id),
    FOREIGN KEY (user_id) REFERENCES forum_users(id)
);

-- Posts table
CREATE TABLE IF NOT EXISTS forum_posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    thread_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (thread_id) REFERENCES forum_threads(id),
    FOREIGN KEY (user_id) REFERENCES forum_users(id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_threads_category ON forum_threads(category_id);
CREATE INDEX IF NOT EXISTS idx_threads_user ON forum_threads(user_id);
CREATE INDEX IF NOT EXISTS idx_posts_thread ON forum_posts(thread_id);
CREATE INDEX IF NOT EXISTS idx_posts_user ON forum_posts(user_id);

-- Insert some sample data
INSERT OR IGNORE INTO forum_users (id, username, password_hash) VALUES
(1, 'admin', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj7/rE8V5Ae'); -- password: admin123

INSERT OR IGNORE INTO forum_categories (id, name, description, sort_order) VALUES
(1, 'General Discussion', 'General forum discussions', 1),
(2, 'Technical Support', 'Get help with technical issues', 2),
(3, 'Feature Requests', 'Suggest new features', 3);

INSERT OR IGNORE INTO forum_threads (id, category_id, user_id, title) VALUES
(1, 1, 1, 'Welcome to Oxibase Forum'),
(2, 2, 1, 'How to get started');

INSERT OR IGNORE INTO forum_posts (thread_id, user_id, content) VALUES
(1, 1, 'Welcome to the Oxibase forum! This is a demonstration of the forum functionality built directly into the database.'),
(2, 1, 'To get started with Oxibase, first install it using cargo, then run the CLI or web server.');