# Interface Contract: DAP WebSocket Proxy

## Endpoint
`ws://<oxibase-host>:<oxibase-port>/workspace/dap-ws`

## Protocol
The WebSocket connection transports raw strings exactly conforming to the Debug Adapter Protocol (DAP), including HTTP-style headers.

Unlike typical WebSocket JSON protocols, this implementation strictly adheres to the DAP specification and requires parsing the `Content-Length` header to read the corresponding JSON payload.

### Client -> Server (Browser to Oxibase)
The client sends standard DAP messages as string payloads over the WebSocket.

```text
Content-Length: 119\r\n\r\n{
  "seq": 1,
  "type": "request",
  "command": "next",
  "arguments": {
    "threadId": 1
  }
}
```

### Server -> Client (Oxibase to Browser)
The server sends standard DAP messages as string payloads.

**Response Example:**
```text
Content-Length: 110\r\n\r\n{
  "seq": 2,
  "type": "response",
  "request_seq": 1,
  "command": "next",
  "success": true
}
```

**Event Example:**
```json
{
  "seq": 3,
  "type": "event",
  "event": "stopped",
  "body": {
    "reason": "breakpoint",
    "threadId": 1
  }
}
```