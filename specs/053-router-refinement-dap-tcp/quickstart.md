# Quickstart: Router Refinement & Standalone DAP TCP

This guide walks you through starting, utilizing, and verifying the refined generic router and the external DAP TCP debugger.

## 1. Verifying Generic Router Refinements

### UTF-8 Decoder Check
Start the database server, open your browser, and run a localized query containing UTF-8 percent-encoded string. For instance:

```bash
curl -G "http://localhost:8080/api/data/users" \
  --data-urlencode "name=eq.Alice á"
```
The server will cleanly decode the hex elements into standard UTF-8 characters and match records successfully.

### Comment-Stair Routing Check
Post a query documented with inline comments to `/workspace/sql` to verify correct query type classification:

```bash
curl -X POST "http://localhost:8080/workspace/sql" \
  -H "Content-Type: application/json" \
  -d '{"query": "-- Get user count\nSELECT COUNT(*) FROM users;"}'
```
The router will ignore the single-line comment and run the SQL query as a SELECT query, returning row results instead of executing a blank action block.

---

## 2. Using Standalone DAP TCP Debugger

To debug Oxibase stored procedures (Rhai or PL/SQL) directly from an external IDE (such as VS Code or Neovim):

### VS Code Setup
Create a new debug profile inside your project's `.vscode/launch.json` file:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "mock",
      "request": "launch",
      "name": "Debug Oxibase Procedure",
      "program": "${workspaceFolder}/procedure_name",
      "stopOnEntry": true,
      "port": 4711,
      "host": "localhost"
    }
  ]
}
```

### Steps to Run
1. Start the Oxibase Database engine on your host machine.
2. Open your procedure file inside VS Code and set breakpoints on the line gutter.
3. Start the debug configuration. VS Code connects to port `4711` and handshakes automatically.
4. Execute the stored procedure (e.g. by posting a payload to `/api/rpc/procedure_name`).
5. VS Code highlights the paused line. You can now inspect variables, view frames, and step over statements!
