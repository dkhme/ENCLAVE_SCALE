# EnclaveScale: Reference Implementation

This repository contains the public code artifact for the paper:
**"EnclaveScale: A Hardware-Assisted Framework for Privacy-Preserving Data Center Power Profiling"**.

EnclaveScale resolves the dichotomy between massive WAN bandwidth costs and centralized trust concentration by deploying attested edge sanitisation. This artifact provides a high-fidelity, production-grade Rust implementation of the system's core cryptographic, differential privacy, and mathematical pipelines.

## Architectural Mapping to Paper Sections

The codebase is modularized to strictly reflect the algorithms and equations detailed in the manuscript:

- **`src/lse.rs`**: Implements the Local Sanitisation Enclave (LSE) Pipeline. Includes plaintext transition counting, DP noise injection, and amortized attestation binding (**Algorithm 1, §4.2**).
- **`src/gae.rs`**: Implements the Global Aggregation Enclave (GAE). Handles TDX quote verification, replay protection via monotonic counters, and capacity-weighted hardware-stratified aggregation (**Algorithm 2, §4.3 & §4.4**).
- **`src/dp.rs`**: Enforces the strict $\ell_2$-sensitivity bound ($\Delta_2 f = 2$) and applies the Gaussian Mechanism with non-negativity projection and row-stochastic normalisation (**§5.1**).
- **`src/crypto.rs`**: Simulates the Intel TDX DCAP quoting interface and implements the per-batch Ed25519 payload hashing constraint ($H = \text{SHA256}(\hat{M}_i \parallel h \parallel \text{timestamp} \parallel b)$) (**§4.3**).
- **`src/telemetry.rs`**: Handles the debounced discretisation of 10-Hz continuous power transients, preventing boundary-straddling hardware manipulation (**§4.2**).
- **`src/grid.rs`**: Calculates the physical microgrid peak-load margin from the probabilistic Markov matrix using spectral gap estimation (**Equation 3, §4.5**).

## Building and Running

### Prerequisites
- Rust toolchain (Edition 2021)
- `cargo` package manager

### Execution
To execute the end-to-end evaluation simulating 20 geo-distributed LSEs performing telemetry ingestion, DP injection, attestation, and GAE aggregation:

```bash
cargo run --release
```

### Expected Output
The harness will output the initialization parameters, the result of the GAE signature verification and anti-replay filter, the end-to-end processing latency, and the resulting Microgrid Peak-Load Margin in Megawatts (MW) derived from the differentially private aggregate matrix.

## Dependencies
- `nalgebra`: High-performance matrix operations and eigenvector estimation.
- `ed25519-dalek`: Cryptographic signatures for per-batch attestation amortisation.
- `sha2`: Payload hashing for execution integrity binding.
- `rand` & `rand_distr`: CSPRNG for Gaussian mechanism noise sampling.