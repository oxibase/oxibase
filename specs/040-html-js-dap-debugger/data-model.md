# Data Model: HTML/JS DAP Debugger

This feature introduces frontend state representations of the DAP protocol.

## Frontend State Entities

### `DAPClient`
The vanilla JavaScript class responsible for the WebSocket connection and protocol state.
- `ws: WebSocket` - The active WebSocket connection.
- `seq: number` - Auto-incrementing sequence number for outgoing requests.
- `pendingRequests: Map<number, {resolve, reject}>` - Tracks requests awaiting a response.
- `listeners: Map<string, Function[]>` - Event listeners for DAP events (e.g., `stopped`, `output`).

### `DebugSession`
The logical state of the current debugging session within the UI.
- `active: boolean` - Whether a session is currently running/attached.
- `paused: boolean` - Whether execution is currently stopped at a breakpoint or step.
- `currentThreadId: number | null` - The ID of the currently focused thread.
- `currentFrameId: number | null` - The ID of the currently focused stack frame.
- `breakpoints: Map<string, Set<number>>` - A map of file URIs to a set of active line numbers with breakpoints.

### `VariableNode` (DOM Representation)
The visual representation of a DAP Variable in the sidebar tree.
- Represented by HTML `<details>` and `<summary>` elements.
- `name: string` - The variable name.
- `value: string` - The variable's stringified value.
- `type: string` - The variable type.
- `variablesReference: number` - If > 0, the variable has children that can be fetched when the `<details>` element is toggled open.