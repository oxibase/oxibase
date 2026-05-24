#!/bin/bash
cargo build --features server
cargo run --features server -- serve --port 8080 &
SERVER_PID=$!
sleep 2

curl -s http://localhost:8080/api/information_schema.tables
echo ""

kill $SERVER_PID
