# Research: Router Refinement & Standalone DAP TCP

## 1. Safe UTF-8 URL Percent Decoder

### Decision
Rewrite `url_decode` to aggregate characters into a raw binary buffer (`Vec<u8>`) instead of casting percent-decoded bytes to `char` individually. Convert to string at the end of decoding.

### Rationale
A percent-encoded UTF-8 character like `á` is sent as `%C3%A1` (two bytes). Casting `%C3` to `char` yields `Ã` (unicode code-point 195) and `%A1` yields `¡` (unicode code-point 161), resulting in corrupted text `"Ã¡"`. By collecting raw bytes in a byte buffer, `%C3` and `%A1` are pushed as bytes `0xC3` and `0xA1` into `Vec<u8>`. Translating using `String::from_utf8_lossy(&bytes)` reconstructs the correct uncorrupted character `"á"`.

### Alternatives Considered
* **Using external URL decoding crate (e.g. `percent-encoding`)**: This would introduce a new dependency. The byte-buffer collector approach is extremely simple (~15 lines of code) and achieves the exact same goal with zero-overhead (YAGNI / Ponytail).

---

## 2. Stripping SQL Comments for Query Type Selection

### Decision
Implement a specialized helper `clean_sql_for_classification` that filters out leading whitespace, single-line SQL comments starting with `--`, and multi-line SQL comments enclosed in `/* ... */`.

### Rationale
A SQL statement documented with top-level comments (e.g. `-- list user data\nSELECT * FROM users`) fails basic `starts_with("SELECT")` checks. This causes the query execution engine to run the SELECT query as an action block via `execute_named()`, returning zero rows and producing blank dashboard screens. Stripping leading comments and spaces before executing the prefix check makes the router completely robust.

### Alternatives Considered
* **Full AST Parsing**: Passing every query to the SQL parser to extract the query statement type. However, parsing has overhead and can fail if there are minor syntax issues. A quick lexical stripper of comments/whitespace runs in $O(N)$ time and is completely robust for routing selection.

---

## 3. Standalone TCP DAP Listener

### Decision
Expose a standalone TCP port listener (port `4711`) inside the server initialization sequence. Spawn a Tokio background task that listens for incoming TCP connections, parses standard DAP packets framed with `Content-Length`, and bridges them directly to the shared `DebugController`.

### Rationale
Traditional editors like VS Code, Neovim, and Sublime Text speak the standard DAP protocol over raw TCP rather than WebSockets. Hosting a standalone TCP listener allows native integration with standard development environments, allowing developers to debug Oxibase procedures directly in their favorite IDEs.

### Alternatives Considered
* **WebSocket proxying**: Requiring developers to run a separate Node/Python WebSocket-to-TCP bridge. This adds operational complexity for the end-user. Direct TCP support inside the engine matches the monolithic and self-contained design principles.
