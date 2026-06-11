# Quickstart: Debugging PL/SQL Procedures via DAP

Oxibase supports native debugging of PL/SQL procedures using the standard Debug Adapter Protocol (DAP).

## Prerequisites
1. Ensure your Oxibase server is started with the DAP server enabled (e.g., passing `--debug-port 4711`).
2. Have a DAP-compatible client like VS Code or Zed.

## Attaching the Debugger

In **VS Code**, create a `launch.json` file to attach to the Oxibase debug port:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Attach to Oxibase PL/SQL",
            "type": "oxibase",
            "request": "attach",
            "port": 4711,
            "host": "localhost"
        }
    ]
}
```

## Setting Breakpoints and Debugging

1. Open the `.sql` file containing your PL/SQL procedure source code in your editor.
2. Click on the left gutter to set a breakpoint on any executable line (e.g., inside the `BEGIN ... END;` block).
3. Start the debug session in your IDE (this connects to the Oxibase DAP server).
4. Execute the procedure in a separate SQL client (e.g., `CALL my_procedure();`).

When the execution reaches the line with your breakpoint:
- The SQL client execution will pause.
- Your IDE will pause on the breakpoint.
- You can inspect local variables in the IDE's variables pane.
- You can use "Step Over", "Step In" (if applicable), and "Continue" commands in your IDE.
