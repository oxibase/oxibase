# Quickstart: Foreign Key Constraints

This feature is internal to the database engine and exposes standard SQL syntax.

## Defining a Foreign Key

```sql
-- During table creation
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR
);

CREATE TABLE posts (
    id INT PRIMARY KEY,
    user_id INT,
    title VARCHAR,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Adding to an existing table
ALTER TABLE posts ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL;
```

## Behavior

- Inserting a post with a `user_id` that doesn't exist in `users` will fail.
- Deleting a user will automatically delete all their posts (if `ON DELETE CASCADE`) or set the post's `user_id` to NULL (if `ON DELETE SET NULL`).