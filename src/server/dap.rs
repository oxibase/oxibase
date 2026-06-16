// Copyright 2025 Oxibase Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::server::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::header,
    response::IntoResponse,
};
use tokio::sync::mpsc;

/// Serves the vanilla JavaScript DAP client
pub async fn serve_dap_client() -> impl IntoResponse {
    let js = include_str!("../bin/workspace/static/js/dap-client.js");
    ([(header::CONTENT_TYPE, "application/javascript")], js)
}

/// Upgrades the connection to a WebSocket for DAP communication
pub async fn dap_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut buffer = String::new();

    // Set up a channel for the DebugController to send events back to the client
    let (tx, mut rx) = mpsc::unbounded_channel::<serde_json::Value>();
    {
        let mut client_tx = state.debug_controller.client_tx.lock().unwrap();
        *client_tx = Some(tx);
    }

    loop {
        tokio::select! {
            Some(msg) = socket.recv() => {
                if let Ok(msg) = msg {
                    match msg {
                        Message::Text(text) => {
                            buffer.push_str(&text);
                            process_dap_buffer(&mut buffer, &mut socket, &state).await;
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                } else {
                    break;
                }
            }
            Some(event) = rx.recv() => {
                if let Ok(json_str) = serde_json::to_string(&event) {
                    if send_dap_message(&mut socket, &json_str).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    // Cleanup
    {
        let mut client_tx = state.debug_controller.client_tx.lock().unwrap();
        *client_tx = None;
    }
}

async fn process_dap_buffer(buffer: &mut String, socket: &mut WebSocket, state: &AppState) {
    loop {
        // Look for the end of the headers
        if let Some(header_end) = buffer.find("\r\n\r\n") {
            let headers = &buffer[..header_end];

            // Extract Content-Length
            let mut content_length: Option<usize> = None;
            for line in headers.split("\r\n") {
                if line.to_lowercase().starts_with("content-length:") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() == 2 {
                        if let Ok(len) = parts[1].trim().parse::<usize>() {
                            content_length = Some(len);
                        }
                    }
                }
            }

            if let Some(len) = content_length {
                let body_start = header_end + 4;
                if buffer.len() >= body_start + len {
                    // We have the full message
                    let payload = &buffer[body_start..body_start + len];

                    // Parse the JSON payload
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(payload) {
                        println!("Received DAP JSON: {}", json);

                        // Pass JSON to internal DebugController here
                        if let Some(type_val) = json.get("type").and_then(|v| v.as_str()) {
                            if type_val == "request" {
                                if let Some(cmd) = json.get("command").and_then(|v| v.as_str()) {
                                    let seq = json.get("seq").and_then(|v| v.as_i64()).unwrap_or(1);

                                    if cmd == "initialize" {
                                        let resp = serde_json::json!({
                                            "seq": 1,
                                            "type": "response",
                                            "request_seq": seq,
                                            "command": "initialize",
                                            "success": true,
                                            "body": {
                                                "supportsConfigurationDoneRequest": true,
                                                "supportsFunctionBreakpoints": true,
                                            }
                                        });
                                        if let Ok(resp_str) = serde_json::to_string(&resp) {
                                            let _ = send_dap_message(socket, &resp_str).await;
                                        }

                                        let event = serde_json::json!({
                                            "seq": 2,
                                            "type": "event",
                                            "event": "initialized"
                                        });
                                        if let Ok(event_str) = serde_json::to_string(&event) {
                                            let _ = send_dap_message(socket, &event_str).await;
                                        }
                                    } else if cmd == "setBreakpoints" {
                                        // Update DebugController breakpoints
                                        if let Some(args) = json.get("arguments") {
                                            if let Some(source) = args.get("source") {
                                                if let Some(path) =
                                                    source.get("path").and_then(|v| v.as_str())
                                                {
                                                    let mut lines = Vec::new();
                                                    if let Some(bps) = args
                                                        .get("breakpoints")
                                                        .and_then(|v| v.as_array())
                                                    {
                                                        for bp in bps {
                                                            if let Some(line) = bp
                                                                .get("line")
                                                                .and_then(|v| v.as_u64())
                                                            {
                                                                lines.push(line as usize);
                                                            }
                                                        }
                                                    }
                                                    println!(
                                                        "DAP setBreakpoints for {} at lines {:?}",
                                                        path, lines
                                                    );
                                                    state
                                                        .debug_controller
                                                        .set_breakpoints(path, lines);
                                                }
                                            }
                                        }

                                        // Reply
                                        let resp = serde_json::json!({
                                            "seq": 1,
                                            "type": "response",
                                            "request_seq": seq,
                                            "command": "setBreakpoints",
                                            "success": true,
                                            "body": {
                                                "breakpoints": []
                                            }
                                        });
                                        if let Ok(resp_str) = serde_json::to_string(&resp) {
                                            let _ = send_dap_message(socket, &resp_str).await;
                                        }
                                    } else if cmd == "continue" {
                                        state
                                            .debug_controller
                                            .resume(crate::common::debug::ResumeAction::Continue);
                                        let resp = serde_json::json!({
                                            "seq": 1,
                                            "type": "response",
                                            "request_seq": seq,
                                            "command": "continue",
                                            "success": true
                                        });
                                        if let Ok(resp_str) = serde_json::to_string(&resp) {
                                            let _ = send_dap_message(socket, &resp_str).await;
                                        }
                                        // Emit continued event
                                        let event = serde_json::json!({
                                            "seq": 2,
                                            "type": "event",
                                            "event": "continued",
                                            "body": {
                                                "threadId": 1,
                                                "allThreadsContinued": true
                                            }
                                        });
                                        if let Ok(event_str) = serde_json::to_string(&event) {
                                            let _ = send_dap_message(socket, &event_str).await;
                                        }
                                    } else if cmd == "next" {
                                        state
                                            .debug_controller
                                            .resume(crate::common::debug::ResumeAction::StepOver);
                                        let resp = serde_json::json!({
                                            "seq": 1,
                                            "type": "response",
                                            "request_seq": seq,
                                            "command": "next",
                                            "success": true
                                        });
                                        if let Ok(resp_str) = serde_json::to_string(&resp) {
                                            let _ = send_dap_message(socket, &resp_str).await;
                                        }
                                    } else if cmd == "disconnect" {
                                        state
                                            .debug_controller
                                            .resume(crate::common::debug::ResumeAction::Disconnect);
                                        let resp = serde_json::json!({
                                            "seq": 1,
                                            "type": "response",
                                            "request_seq": seq,
                                            "command": "disconnect",
                                            "success": true
                                        });
                                        if let Ok(resp_str) = serde_json::to_string(&resp) {
                                            let _ = send_dap_message(socket, &resp_str).await;
                                        }
                                    } else if cmd == "threads" {
                                        let resp = serde_json::json!({
                                            "seq": 1,
                                            "type": "response",
                                            "request_seq": seq,
                                            "command": "threads",
                                            "success": true,
                                            "body": {
                                                "threads": [{"id": 1, "name": "Main Thread"}]
                                            }
                                        });
                                        if let Ok(resp_str) = serde_json::to_string(&resp) {
                                            let _ = send_dap_message(socket, &resp_str).await;
                                        }
                                    } else if cmd == "stackTrace" {
                                        let resp = serde_json::json!({
                                            "seq": 1,
                                            "type": "response",
                                            "request_seq": seq,
                                            "command": "stackTrace",
                                            "success": true,
                                            "body": {
                                                "stackFrames": [
                                                    {"id": 1, "name": "Execution Context", "line": 0, "column": 0}
                                                ]
                                            }
                                        });
                                        if let Ok(resp_str) = serde_json::to_string(&resp) {
                                            let _ = send_dap_message(socket, &resp_str).await;
                                        }
                                    } else if cmd == "scopes" {
                                        let resp = serde_json::json!({
                                            "seq": 1,
                                            "type": "response",
                                            "request_seq": seq,
                                            "command": "scopes",
                                            "success": true,
                                            "body": {
                                                "scopes": [
                                                    {"name": "Locals", "variablesReference": 1, "expensive": false},
                                                    {"name": "Globals", "variablesReference": 2, "expensive": false}
                                                ]
                                            }
                                        });
                                        if let Ok(resp_str) = serde_json::to_string(&resp) {
                                            let _ = send_dap_message(socket, &resp_str).await;
                                        }
                                    } else if cmd == "variables" {
                                        let vars_ref = json
                                            .get("arguments")
                                            .and_then(|a| a.get("variablesReference"))
                                            .and_then(|v| v.as_i64())
                                            .unwrap_or(0);
                                        let mut vars = Vec::new();

                                        {
                                            let pause_state =
                                                state.debug_controller.pause_mutex.lock().unwrap();
                                            if vars_ref == 1 {
                                                if let Some(locals) = &pause_state.current_locals {
                                                    if let Some(obj) = locals.as_object() {
                                                        for (k, v) in obj {
                                                            vars.push(serde_json::json!({
                                                                "name": k,
                                                                "value": v.to_string(),
                                                                "type": "object",
                                                                "variablesReference": 0
                                                            }));
                                                        }
                                                    }
                                                }
                                            } else if vars_ref == 2 {
                                                if let Some(globals) = &pause_state.current_globals
                                                {
                                                    if let Some(obj) = globals.as_object() {
                                                        for (k, v) in obj {
                                                            vars.push(serde_json::json!({
                                                                "name": k,
                                                                "value": v.to_string(),
                                                                "type": "object",
                                                                "variablesReference": 0
                                                            }));
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        let resp = serde_json::json!({
                                            "seq": 1,
                                            "type": "response",
                                            "request_seq": seq,
                                            "command": "variables",
                                            "success": true,
                                            "body": {
                                                "variables": vars
                                            }
                                        });
                                        if let Ok(resp_str) = serde_json::to_string(&resp) {
                                            let _ = send_dap_message(socket, &resp_str).await;
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        eprintln!("Failed to parse DAP JSON: {}", payload);
                    }

                    // Remove processed message from buffer
                    *buffer = buffer[body_start + len..].to_string();
                    continue; // Check if there's another message in the buffer
                }
            }
        }
        break; // Need more data
    }
}

async fn send_dap_message(socket: &mut WebSocket, json_str: &str) -> Result<(), axum::Error> {
    let payload = format!("Content-Length: {}\r\n\r\n{}", json_str.len(), json_str);
    socket.send(Message::Text(payload.into())).await
}
