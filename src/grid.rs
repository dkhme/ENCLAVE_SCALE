use nalgebra::{DMatrix, ComplexField};

/// Calculates the microgrid peak-load provisioning margin (Section 4.5)
pub fn calculate_peak_margin(matrix: &DMatrix<f64>, power_profile: &[f64], total_capacity: f64, kappa: f64) -> f64 {
    let n = matrix.nrows();
    
    // 1. Steady-state distribution (Left principal eigenvector for lambda=1)
    // Using power iteration for stable, deterministic approximation of the Markov chain's stationary distribution.
    let mut pi = DMatrix::from_element(1, n, 1.0 / n as f64);
    for _ in 0..200 {
        pi = &pi * matrix;
    }

    // 2. Expected Baseline Power E[P]
    let mut expected_power = 0.0;
    for i in 0..n {
        expected_power += pi[(0, i)] * power_profile[i];
    }
    expected_power *= total_capacity;

    // 3. Spectral Gap estimation
    // Using nalgebra's ComplexEigen for full eigendecomposition to accurately find lambda_2.
    let eig = matrix.complex_eigenvalues();
    let mut evals: Vec<f64> = eig.into_iter().map(|c| c.norm1()).collect();
    evals.sort_by(|a, b| b.partial_cmp(a).unwrap());
    
    // lambda_2 is the second largest eigenvalue (if n >= 2, else 0.0)
    let lambda_2_approx = if evals.len() >= 2 { evals[1].min(0.99_f64) } else { 0.0_f64 };
    let gamma = 1.0_f64 - lambda_2_approx;

    // 4. Facility Peak Load Equation (Equation 3)
    let p_max = power_profile.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let margin = expected_power + total_capacity * p_max * kappa * (1.0_f64 / gamma).sqrt();
    
    // Cap the margin to the physical ceiling
    let physical_ceiling = total_capacity * p_max;
    if margin > physical_ceiling {
        physical_ceiling
    } else {
        margin
    }
}
