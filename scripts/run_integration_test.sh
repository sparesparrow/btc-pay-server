
#!/bin/bash

echo "Starting BTC Pay Server..."
cargo run --bin btc-pay-server &
SERVER_PID=$!

# Wait for server to initialize
sleep 3

echo "Running client test..."
cargo run --bin client

echo "Stopping server..."
kill $SERVER_PID

echo "Integration test completed"
