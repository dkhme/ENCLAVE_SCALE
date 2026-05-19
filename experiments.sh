#!/bin/bash

echo "============================================================"
echo " EnclaveScale: 20-Node Multi-Region TDX Distributed Execution"
echo "============================================================"

# Start GAE
cargo run --release -- --role gae &
GAE_PID=$!

sleep 2 # Wait for GAE to bind

echo "Starting 20 LSE nodes..."
for i in {1..20}; do
    # Distribute hardware profiles for the simulation
    if [ $i -le 10 ]; then
        HW="H100"
    elif [ $i -le 15 ]; then
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
