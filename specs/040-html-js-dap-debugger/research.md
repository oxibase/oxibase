# Research & Decisions: HTML/JS DAP Debugger

## 1. CodeMirror 6 Integration
- **Decision**: Use CodeMirror 6 for the code editor, leveraging `@codemirror/view`'s gutter system to display and interact with breakpoints.
- **Rationale**: CodeMirror 6 is modular and works well in a vanilla JS environment without requiring heavy frontend frameworks like React. The gutter extension allows us to render custom DOM elements (like red dots for breakpoints) and listen for click events on line numbers to toggle breakpoints.
- **Alternatives considered**: Monaco Editor was considered but is much heavier (~3-4MB) and more complex to bundle in a vanilla setup without a build tool like webpack/vite. CodeMirror is lightweight and sufficient.

## 2. WebSocket Protocol & DAP Framing
- **Decision**: The WebSocket connection (`/workspace/dap-ws`) will transmit standard DAP messages, including the HTTP-like `Content-Length` headers, exactly as specified by the Debug Adapter Protocol.
- **Rationale**: Strict compliance with the DAP specification ensures that the proxy is reusable with standard DAP clients if ever needed, and respects the user's explicit directive to follow the standard even over WebSockets. The frontend vanilla JS client will implement a small buffer parser to extract the JSON payload based on the `Content-Length` header.
- **Alternatives considered**: Stripping headers and sending pure JSON over WebSockets. Rejected due to explicit instruction to follow the standard DAP specification.

## 3. State Persistence with Unpoly
- **Decision**: Wrap the debugger workspace (editor + sidebar) in an element with the `up-keep` attribute. Ensure the `DAPClient` and `CodeMirror` instances are attached to a stable DOM node or global state that Unpoly preserves.
- **Rationale**: Unpoly's `[up-keep]` attribute preserves the DOM element and its state across fragment updates. This is crucial to prevent the WebSocket connection from dropping and the CodeMirror instance from being destroyed when the user navigates between the "Compute" and "Data" tabs in the workspace.
- **Alternatives considered**: Storing state in `sessionStorage` and reconnecting on every page load. Rejected because reconnecting drops the active debug session context in the backend and causes a jarring UX.