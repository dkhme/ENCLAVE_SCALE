#!/bin/bash
# ==============================================================================
# EnclaveScale: GCP TDX Cluster Provisioning Script (32 Nodes, 4 Regions)
# ==============================================================================
# This script strictly replicates the evaluation environment described in §7
# of the paper. It provisions 32 Intel TDX Confidential VMs on GCP C3 instances.
#
# PREREQUISITES:
# - A valid Google Cloud Project with billing enabled.
# - Quota: 128x C3 vCPUs distributed across the 4 specified regions.
# - 'gcloud' CLI installed and authenticated (`gcloud auth login`).
# ==============================================================================

set -e

echo "Provisioning 32-Node Intel TDX Cluster across 4 regions..."

# The 4 regions evaluated in the paper, mapped to their TDX-available zones
declare -A REGIONS
REGIONS=(
    ["us-central1"]="us-central1-a"
    ["us-east5"]="us-east5-a"
    ["europe-west4"]="europe-west4-c"
    ["asia-southeast1"]="asia-southeast1-b"
)

# Create a dedicated VPC firewall rule to allow GAE/LSE communication (port 8080)
echo "Setting up VPC firewall rules for EnclaveScale..."
gcloud compute firewall-rules create enclavescale-allow-tcp-8080 \
    --allow tcp:8080 \
    --description="Allow LSEs to communicate with GAE" || true

TOTAL_CREATED=0

for REGION in "${!REGIONS[@]}"; do
    ZONE="${REGIONS[$REGION]}"
    echo "============================================================"
    echo " Provisioning 8 LSE nodes in $REGION ($ZONE)"
    echo "============================================================"
    
    for i in {1..8}; do
        NODE_NAME="enclavescale-lse-${REGION}-${i}"
        
        # Note: --on-host-maintenance=TERMINATE is required for TDX on GCP.
        # Note: NVMe interface is required for C3 Confidential Computing.
        gcloud compute instances create "$NODE_NAME" \
            --machine-type=c3-standard-4 \
            --zone="$ZONE" \
            --confidential-compute-type=TDX \
            --on-host-maintenance=TERMINATE \
            --image-family=ubuntu-2404-lts-amd64 \
            --image-project=ubuntu-os-cloud \
            --tags=enclavescale-node \
            --async
            
        TOTAL_CREATED=$((TOTAL_CREATED + 1))
    done
done

echo "============================================================"
echo " Issued creation commands for $TOTAL_CREATED nodes."
echo " Use 'gcloud compute instances list --filter=\"tags:enclavescale-node\"' to monitor status."
echo " Once all nodes are RUNNING, proceed to orchestrate_eval.sh."
echo "============================================================"
