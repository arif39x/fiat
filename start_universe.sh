#!/bin/bash


echo "Starting Uclid Microservices..."

echo "Booting Compiler (Python)..."
cd compiler
source venv/bin/activate
uvicorn api:app --port 8081 &
PID_MATH=$!
cd ..

sleep 1

echo "Booting Server (Go)..."
cd server
go run . &
PID_SERVER=$!
cd ..

sleep 2

echo "Booting Client (Rust)..."
cd client
cargo run --release


echo "Shutting down the universe..."
kill $PID_MATH
kill $PID_SERVER

echo "Uclid Offline."
