#!/bin/bash
# ==============================================================================
# EnclaveScale: GCP Evaluation Orchestration Script
# ==============================================================================
# This script orchestrates the execution of the EnclaveScale pipeline across
# the 32 provisioned GCP TDX VMs.
#
# USAGE: ./orchestrate_eval.sh <GAE_EXTERNAL_IP>
# ==============================================================================

if [ -z "$1" ]; then
    echo "Error: Must provide the external IP of the GAE node."
    echo "Usage: ./orchestrate_eval.sh <GAE_EXTERNAL_IP>"
    exit 1
fi

GAE_IP=$1
echo "Orchestrating evaluation targeting GAE at $GAE_IP:8080..."

# Retrieve all LSE nodes
NODES=$(gcloud compute instances list --filter="tags:enclavescale-node" --format="value(name,zone)")

INDEX=1
while read -r NODE_INFO; do
    if [ -z "$NODE_INFO" ]; then continue; fi
    
    NODE_NAME=$(echo "$NODE_INFO" | awk '{print $1}')
    NODE_ZONE=$(echo "$NODE_INFO" | awk '{print $2}')
    
    # Stratify hardware profiles matching the paper's distribution (11 H100, 11 A100, 10 L4)
    if [ $INDEX -le 11 ]; then
        HW="H100"
    elif [ $INDEX -le 22 ]; then
        HW="A100"
    else
        HW="L4"
    fi

    echo "Deploying to $NODE_NAME in $NODE_ZONE (Profile: $HW)..."
    
    # 1. Install Rust
    # 2. Clone/Copy the artifact (Assuming the repo is publicly accessible or copied via scp)
    # 3. Build and Run
    gcloud compute ssh "$NODE_NAME" --zone="$NODE_ZONE" --command="
        if ! command -v cargo &> /dev/null; then
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source \$HOME/.cargo/env
        fi
        
        if [ ! -d \"enclavescale-artifact\" ]; then
            git clone https://github.com/anonymous/enclavescale-artifact.git || true
        fi
        
        cd enclavescale-artifact
        cargo build --release
        
        # Run LSE in background, outputting to log
        nohup cargo run --release -- --role lse $HW --gae-ip $GAE_IP:8080 > lse.log 2>&1 &
    " &
    
    INDEX=$((INDEX + 1))
done <<< "$NODES"

echo "All LSE nodes instructed to begin streaming. Check GAE logs for incoming attested telemetry."
