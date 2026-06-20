# Quickstart: fix-rhai-debugger

This guide demonstrates how to test and verify the corrected Rhai debugger behavior.

## 1. Verify standard compilation

Ensure the project builds cleanly:
```bash
cargo check --features rhai
```

## 2. Running Debugger Tests

To run the dedicated debugger integration tests (once implemented):
```bash
cargo nextest run --test rhai_scripting_test --features rhai
```

## 3. Web UI Debugging Demonstration

1. Start the Oxibase workstation web server:
   ```bash
   cargo run --bin oxibase --features "cli server"
   ```
2. Navigate to the web workstation: `http://localhost:8080/workspace`
3. Click on the **Compute** tab, and select a Rhai function or procedure.
4. Click on the line gutter in the CodeMirror editor to set a breakpoint.
5. Click **Run / Debug**.
6. Execution will now pause at the breakpoint, and local variables will be displayed in the sidebar.
7. Click **Continue** or **Step Over** to control execution flow.
