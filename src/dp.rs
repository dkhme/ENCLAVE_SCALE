use rand::Rng;
use rand_distr::{Normal, Distribution};
use nalgebra::DMatrix;

/// Global l2-sensitivity derived in Section 5.1: 
/// Modifying one 0.1s sample alters up to three transition counts (worst case -2, +1, +1), yielding sqrt((-2)^2+1^2+1^2) = sqrt(6) ≈ 2.4494897.
pub const DELTA_2_F: f64 = 2.4494897;

/// Implements the Gaussian Mechanism for transition matrices (Section 4.2, Algorithm 1)
pub fn apply_gaussian_noise(
    matrix: &mut DMatrix<f64>,
    epsilon: f64,
    delta: f64,
) {
    // Classical approximation: sigma = Delta_2 f * sqrt(2 * ln(1.25 / delta)) / epsilon
    let sigma = DELTA_2_F * (2.0 * (1.25_f64 / delta).ln()).sqrt() / epsilon;
    let normal = Normal::new(0.0, sigma).expect("Invalid DP normal distribution parameters");
    
    // NOTE: For production TDX deployments, this must use the hardware RDSEED instruction 
    // to prevent hypervisor rollback attacks (see Section 4.3). This artifact uses OsRng 
    // to permit cross-platform simulation and reproducibility.
    let mut rng = rand::rngs::OsRng;

    // Inject Gaussian noise and apply noise-aware thresholding (Section 4.2)
    // Threshold tau_noise = \Phi^{-1}(0.95) * sigma ≈ 1.645 * sigma
    let tau_noise = 1.645 * sigma;
    for val in matrix.iter_mut() {
        let noise = normal.sample(&mut rng);
        let noised_val = *val + noise;
        if noised_val < tau_noise {
            *val = 0.0;
        } else {
            *val = noised_val;
        }
    }

    // Row-stochastic normalisation (deterministic post-processing)
    for mut row in matrix.row_iter_mut() {
        let sum: f64 = row.sum();
        if sum > 0.0 {
            row /= sum;
        } else {
            // Fallback to uniform distribution if noise zeroes out the entire row
            let cols = row.ncols() as f64;
            for v in row.iter_mut() { *v = 1.0 / cols; }
        }
    }
}
