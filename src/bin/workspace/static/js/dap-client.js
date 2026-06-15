// dap-client.js

export class DAPClient {
    constructor(url) {
        this.url = url;
        this.ws = null;
        this.seq = 1;
        this.pendingRequests = new Map();
        this.listeners = new Map();
        this.buffer = "";
    }

    connect() {
        this.ws = new WebSocket(this.url);
        
        this.ws.onopen = () => {
            console.log("[DAP] Connected to", this.url);
        };

        this.ws.onmessage = (event) => {
            this.buffer += event.data;
            this.processBuffer();
        };

        this.ws.onerror = (err) => {
            console.error("[DAP] WebSocket Error:", err);
        };

        this.ws.onclose = () => {
            console.log("[DAP] Disconnected");
        };
    }

    processBuffer() {
        while (true) {
            const headerEnd = this.buffer.indexOf("\r\n\r\n");
            if (headerEnd === -1) break; // Need more data

            const headers = this.buffer.substring(0, headerEnd);
            let contentLength = null;

            const lines = headers.split("\r\n");
            for (const line of lines) {
                if (line.toLowerCase().startsWith("content-length:")) {
                    const parts = line.split(":");
                    if (parts.length === 2) {
                        contentLength = parseInt(parts[1].trim(), 10);
                    }
                }
            }

            if (contentLength === null) {
                console.error("[DAP] Missing Content-Length header. Resetting buffer.");
                this.buffer = "";
                break;
            }

            const bodyStart = headerEnd + 4;
            if (this.buffer.length >= bodyStart + contentLength) {
                const payload = this.buffer.substring(bodyStart, bodyStart + contentLength);
                this.buffer = this.buffer.substring(bodyStart + contentLength);

                try {
                    const message = JSON.parse(payload);
                    this.handleMessage(message);
                } catch (e) {
                    console.error("[DAP] Error parsing JSON payload:", e);
                }
            } else {
                break; // Need more data for the full payload
            }
        }
    }

    handleMessage(message) {
        if (message.type === "response") {
            const requestSeq = message.request_seq;
            if (this.pendingRequests.has(requestSeq)) {
                const { resolve, reject } = this.pendingRequests.get(requestSeq);
                this.pendingRequests.delete(requestSeq);
                if (message.success) {
                    resolve(message);
                } else {
                    reject(message);
                }
            }
        } else if (message.type === "event") {
            this.emit(message.event, message);
        }
    }

    sendRequest(command, args = {}) {
        return new Promise((resolve, reject) => {
            if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
                reject(new Error("WebSocket is not connected"));
                return;
            }

            const requestSeq = this.seq++;
            const message = {
                seq: requestSeq,
                type: "request",
                command,
                arguments: args
            };

            this.pendingRequests.set(requestSeq, { resolve, reject });

            const jsonStr = JSON.stringify(message);
            const payload = `Content-Length: ${jsonStr.length}\r\n\r\n${jsonStr}`;
            
            this.ws.send(payload);
        });
    }

    on(event, callback) {
        if (!this.listeners.has(event)) {
            this.listeners.set(event, []);
        }
        this.listeners.get(event).push(callback);
    }

    off(event, callback) {
        if (this.listeners.has(event)) {
            const callbacks = this.listeners.get(event).filter(cb => cb !== callback);
            this.listeners.set(event, callbacks);
        }
    }

    emit(event, data) {
        if (this.listeners.has(event)) {
            for (const callback of this.listeners.get(event)) {
                try {
                    callback(data);
                } catch (e) {
                    console.error(`[DAP] Error in event listener for ${event}:`, e);
                }
            }
        }
    }
    
    disconnect() {
        if (this.ws) {
            this.ws.close();
        }
    }
}
