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

### Execution
To execute the end-to-end evaluation simulating 32 geo-distributed LSEs performing telemetry ingestion, DP injection, attestation, and GAE aggregation:

```bash
./experiments.sh
```

### Expected Output
The harness will output the initialization parameters, the result of the GAE signature verification and anti-replay filter, the end-to-end processing latency, and the resulting Microgrid Peak-Load Margin in Megawatts (MW) derived from the differentially private aggregate matrix.

## Dependencies
- `nalgebra`: High-performance matrix operations and eigenvector estimation.
- `ed25519-dalek`: Cryptographic signatures for per-batch attestation amortisation.
- `sha2`: Payload hashing for execution integrity binding.
- `rand` & `rand_distr`: CSPRNG for Gaussian mechanism noise sampling.