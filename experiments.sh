#!/bin/bash

echo "============================================================"
echo " EnclaveScale: 32-Node Multi-Region TDX Distributed Execution"
echo "============================================================"

# Start GAE
cargo run --release -- --role gae &
GAE_PID=$!

sleep 2 # Wait for GAE to bind

echo "Starting 32 LSE nodes..."
for i in {1..32}; do
    # Distribute hardware profiles for the simulation
    if [ $i -le 11 ]; then
        HW="H100"
    elif [ $i -le 22 ]; then
        HW="A100"
    else
        HW="L4"
    fi
    
    cargo run --release -- --role lse $HW &
done

echo "Waiting for all submissions..."
wait

echo "Shutting down GAE..."
kill $GAE_PID
