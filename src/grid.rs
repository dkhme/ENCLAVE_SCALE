use nalgebra::DMatrix;

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
    let mut evals: Vec<f64> = eig.into_iter().map(|c| c.norm()).collect();
    evals.sort_by(|a, b| b.partial_cmp(a).unwrap());
    
    // lambda_2 is the second largest eigenvalue (if n >= 2, else 0.0)
    let lambda_2_approx = if evals.len() >= 2 { evals[1].min(0.99) } else { 0.0 };
    let gamma = 1.0 - lambda_2_approx;

    // 4. Facility Peak Load Equation (Equation 3)
    let margin = expected_power * (1.0 + kappa * (1.0 / gamma).sqrt());
    margin
}
