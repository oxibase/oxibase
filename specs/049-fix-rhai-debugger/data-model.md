# Data Model & State Transitions: fix-rhai-debugger

This bugfix introduces no changes to the physical storage layout, schema, or system tables. It purely refactors the state transitions of the Rhai scripting engine execution loop during interactive debugging.

## Debugger State Transitions

The execution thread of a Rhai script transitions through the following states:

```text
               [ Start Execution ]
                        │
                        ▼
                [ State: Running ]
              (Status = StepInto)
                        │
                        ├─► [ Statement Eval ]
                        │          │
                        │          ▼
                        │     Breakpoint?
                        │     ├── Yes ──► [ Pause Thread (Condvar) ]
                        │     │                    │
                        │     │             ResumeAction?
                        │     │             ├── Continue ──► Status = StepInto
                        │     │             ├── StepOver ──► Status = StepOver
                        │     │             └── Disconnect ─► Status = Continue
                        │     └── No ───► Ok(Status) ──► Continue Loop
                        │
                        ▼
               [ End Execution ]
```

## Debugger Command Mapping

The transition decisions from the paused state depend on the DAP/DebugController inputs:

| Debugger Action | ResumeAction (Controller) | Rhai DebuggerCommand |
|-----------------|---------------------------|-----------------------|
| Continue        | `Continue`                | `StepInto` (to check next lines) |
| Step Over       | `StepOver`                | `StepOver`            |
| Disconnect/Stop | `Disconnect`              | `Continue`            |
