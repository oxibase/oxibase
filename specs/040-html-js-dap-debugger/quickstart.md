# Quickstart: HTML/JS DAP Debugger Frontend

This guide explains how to spin up the web workspace and test the DAP frontend integration.

## 1. Start the Oxibase Workspace

Run the Oxibase workspace server locally:

```bash
cargo run --bin workspace
```

This will start the HTTP server (typically on `http://127.0.0.1:3000` or similar, depending on your configuration).

## 2. Access the Workspace

Open your browser and navigate to the workspace URL (e.g., `http://127.0.0.1:3000/workspace`).

Navigate to the "Compute" or "SQL" tab where the CodeMirror editor is displayed.

## 3. Test the WebSocket Connection

1. Open your browser's Developer Tools (F12).
2. Go to the "Network" tab and filter by "WS" (WebSockets).
3. Reload the page. You should see a successful WebSocket connection to `/workspace/dap-ws`.
4. Check the "Console" tab. The `DAPClient` should log initialization messages.

## 4. Test Breakpoints and Stepping

1. In the CodeMirror editor, write a simple PL/SQL, Rhai, or Python procedure.
2. Click on the line number gutter (e.g., line 3). A red breakpoint indicator should appear.
3. In the browser console, you should see a log indicating a `setBreakpoints` request was sent.
4. Execute the procedure (e.g., by clicking "Run" or pressing Cmd+Enter).
5. The execution should pause at the breakpoint. The UI should enter a "paused" state (controls enabled).
6. Click the "Step Over" button in the UI. The execution should move to the next line.
7. Observe the variables in the sidebar updating with the current scope's state.