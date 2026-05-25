# EnclaveScale: Reference Implementation

This repository contains the public code artifact for the paper:
**"EnclaveScale: A Hardware-Assisted Framework for Privacy-Preserving Data Center Power Profiling"**.

EnclaveScale resolves the dichotomy between massive WAN bandwidth costs and centralized trust concentration by deploying attested edge sanitisation. This artifact provides a high-fidelity, production-grade Rust implementation of the system's core cryptographic, differential privacy, and mathematical pipelines.

## Architectural Mapping to Paper Sections

The codebase is modularized to strictly reflect the algorithms and equations detailed in the manuscript:

- **`src/lse.rs`**: Implements the Local Sanitisation Enclave (LSE) Pipeline. Includes plaintext transition counting, DP noise injection, and amortized attestation binding (**Algorithm 1, §4.2**).
- **`src/gae.rs`**: Implements the Global Aggregation Enclave (GAE). Handles TDX quote verification, replay protection via monotonic counters, and capacity-weighted hardware-stratified aggregation (**Algorithm 2, §4.3 & §4.4**).
- **`src/dp.rs`**: Enforces the strict $\ell_2$-sensitivity bound ($\Delta_2 f = \sqrt{6}$) and applies the Gaussian Mechanism with non-negativity projection and row-stochastic normalisation (**§5.1**).
- **`src/crypto.rs`**: Simulates the Intel TDX DCAP quoting interface and implements the per-batch Ed25519 payload hashing constraint ($H = \text{SHA256}(\hat{M}_i \parallel h \parallel \text{timestamp} \parallel b)$) (**§4.3**).
- **`src/telemetry.rs`**: Handles the debounced discretisation of 10-Hz continuous power transients, preventing boundary-straddling hardware manipulation (**§4.2**).
- **`src/grid.rs`**: Calculates the physical microgrid peak-load margin from the probabilistic Markov matrix using spectral gap estimation (**Equation 3, §4.5**).

## Reproducing the DP-Utility Pareto Frontier
As demonstrated in **§7.4**, the system supports a tunable Pareto frontier for infrastructure operators:
*   **High-Utility (Default):** $T=60$ batches. Yields $\varepsilon_{\text{epoch}} = 8.8$ and $4.2$ MW error.
*   **Strong-Privacy:** $T=6$ batches. Yields $\varepsilon_{\text{epoch}} = 2.1$ and $\sim 5.6$ MW error.
Because the per-batch noise scale ($\varepsilon=1$ per batch) is identical across both configurations, adjusting the epoch length $T$ directly controls the formal sequential composition bound without requiring algorithmic modifications to the LSE.

## First-Mile Authentication (SPDM)
As discussed in **§8**, to close the "first-mile gap", our GCP prototype validated a hardware-rooted mutual TLS (mTLS) session using `spdm-emu` between a software-emulated BMC and the LSE's TLS endpoint. While the heavy `spdm-emu` suite is not bundled in this minimal repository, the network bindings in `src/lse.rs` are structured to accept an SPDM-authenticated socket stream with negligible protocol overhead.

## Building and Running

### Prerequisites
- Rust toolchain (Edition 2021)
- `cargo` package manager

### 1. Local Simulation (No Cloud Required)
To execute a local simulation of the 32-node distributed topology across 4 regions:

```bash
./experiments.sh
```

### 2. Full GCP TDX Reproduction (Reviewer Cloud Evaluation)
For reviewers wishing to rigorously replicate the exact production environment detailed in **§7**, we provide full Infrastructure-as-Code (IaC) deployment scripts in the `gcp_deployment/` directory.

**Requirements:**
- A Google Cloud Project with billing enabled.
- Compute Quota: $128\times$ C3 vCPUs distributed across `us-central1`, `us-east5`, `europe-west4`, and `asia-southeast1`.

**Steps:**
1. Execute `./gcp_deployment/provision_tdx_cluster.sh` to automatically provision 32 `c3-standard-4` Confidential VMs with Intel TDX enabled (`--confidential-compute-type=TDX`).
2. Start the GAE aggregator on a central node: `cargo run --release -- --role gae`. Note its external IP.
3. Orchestrate the evaluation across the 32 nodes: `./gcp_deployment/orchestrate_eval.sh <GAE_EXTERNAL_IP>`.

### Expected Output
The harness will output the initialization parameters, the result of the GAE signature verification and anti-replay filter, the end-to-end processing latency, and the resulting Microgrid Peak-Load Margin in Megawatts (MW) derived from the differentially private aggregate matrix.

## Dependencies
- `nalgebra`: High-performance matrix operations and eigenvector estimation.
- `ed25519-dalek`: Cryptographic signatures for per-batch attestation amortisation.
- `sha2`: Payload hashing for execution integrity binding.
- `rand` & `rand_distr`: CSPRNG for Gaussian mechanism noise sampling.