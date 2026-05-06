use oxibase::api::Database;
use oxibase::server::create_router;

#[tokio::main]
async fn main() {
    let db = Database::open("memory://").unwrap();
    let app = create_router(db.clone());
    
    // Insert setup data
    db.execute("INSERT INTO templates.source (name, content) VALUES ('layout.html', '<!DOCTYPE html><html><head><title>Oxibase CMS</title><style>body { font-family: sans-serif; margin: 40px; } h1 { color: #333; } ul { padding-left: 20px; } .user-card { border: 1px solid #ccc; padding: 10px; margin-bottom: 10px; border-radius: 4px; }</style></head><body><header><h1>Oxibase Dynamic Pages</h1><nav><a href=\"/\">Home</a> | <a href=\"/users\">Users</a></nav></header><hr><main>{% block content %}{% endblock %}</main></body></html>')", ()).unwrap();
    
    db.execute("INSERT INTO templates.source (name, content) VALUES ('index.html', '{% extends \"layout.html\" %}{% block content %}<h2>Welcome to the Home Page!</h2><p>This page is served entirely from the <code>templates.source</code> table in the database.</p><p>Try visiting the <a href=\"/users\">Users</a> page to see dynamic data injection.</p>{% endblock %}')", ()).unwrap();
    
    db.execute("INSERT INTO templates.source (name, content) VALUES ('users.html', '{% extends \"layout.html\" %}{% block content %}<h2>Users List</h2><p>This page fetches dynamic data from the <code>users</code> table.</p>{% if data %}{% for user in data %}<div class=\"user-card\"><strong>ID:</strong> {{ user.id }} <br><strong>Name:</strong> {{ user.name }} <br><strong>Role:</strong> {{ user.role }}</div>{% endfor %}{% else %}<p>No users found in the database.</p>{% endif %}{% endblock %}')", ()).unwrap();

    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, role TEXT)", ()).unwrap();
    db.execute("INSERT INTO users (id, name, role) VALUES (1, 'Alice', 'Admin')", ()).unwrap();
    db.execute("INSERT INTO users (id, name, role) VALUES (2, 'Bob', 'Editor')", ()).unwrap();
    db.execute("INSERT INTO users (id, name, role) VALUES (3, 'Charlie', 'Viewer')", ()).unwrap();

    db.execute("INSERT INTO routes.definitions (method, path, template_name, context_query) VALUES ('GET', '/', 'index.html', NULL)", ()).unwrap();
    db.execute("INSERT INTO routes.definitions (method, path, template_name, context_query) VALUES ('GET', '/users', 'users.html', 'SELECT id, name, role FROM users ORDER BY id')", ()).unwrap();

    let addr = "127.0.0.1:8080";
    println!("Starting test server on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
