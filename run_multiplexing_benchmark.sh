#!/bin/bash

echo "============================================================"
echo " EnclaveScale: Multi-Session Multiplexing Benchmark"
echo "============================================================"
echo "Building artifact..."
cargo build --release

echo ""
echo "Sweeping concurrent SPDM sessions K in {1, 16, 64, 256, 1024}..."

for K in 1 16 64 256 1024; do
    ./target/release/enclavescale --role lse-benchmark $K
    sleep 1
done

echo "============================================================"
echo "Benchmark Complete."
echo "Use these empirical results to update Section 7.1 if necessary."
