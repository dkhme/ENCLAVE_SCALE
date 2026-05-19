use std::collections::HashMap;
use nalgebra::DMatrix;
use ed25519_dalek::Verifier;
use crate::lse::AttestedSubmission;
use crate::crypto::hash_payload;

pub struct GlobalAggregationEnclave {
    // State tracking for Replay Protection (Algorithm 2)
    max_b_accepted: HashMap<[u8; 32], u64>,
    
    // Hardware-stratified global models (Section 4.4)
    pub global_models: HashMap<String, DMatrix<f64>>,
    pub capacity_weights: HashMap<String, f64>,
}

impl GlobalAggregationEnclave {
    pub fn new() -> Self {
        Self {
            max_b_accepted: HashMap::new(),
            global_models: HashMap::new(),
            capacity_weights: HashMap::new(),
        }
    }

    /// Implements Algorithm 2: GAE Quote Verification and Aggregation
    pub fn verify_and_aggregate(&mut self, sub: &AttestedSubmission, capacity: f64) -> bool {
        // 1. Verify DCAP q_init and PCK chain (Simulated)
        if !sub.q_init.starts_with("TDX_DCAP_QUOTE_BINDING_") {
            return false;
        }

        let vk_bytes = sub.vk.to_bytes();

        // 2. Verify Strict Monotonic Counter (Replay Protection, Section 3.3)
        let last_b = self.max_b_accepted.get(&vk_bytes).copied().unwrap_or(0);
        if sub.batch_counter <= last_b {
            return false; // Replay / Stale Submission detected
        }

        // 3. Verify Ed25519 Signature over Payload
        let payload = hash_payload(&sub.matrix, &sub.hardware, sub.timestamp, sub.batch_counter);
        if sub.vk.verify(&payload, &sub.signature).is_err() {
            return false; // Forgery detected
        }

        // Accept and update counter
        self.max_b_accepted.insert(vk_bytes, sub.batch_counter);

        // 4. Capacity-Weighted Hardware-Stratified Aggregation (Equation 2, Section 4.4)
        let model = self.global_models.entry(sub.hardware.clone())
            .or_insert_with(|| DMatrix::zeros(sub.matrix.nrows(), sub.matrix.ncols()));
        let total_cap = self.capacity_weights.entry(sub.hardware.clone()).or_insert(0.0);
        
        *total_cap += capacity;
        let weight = capacity / *total_cap;
        
        // Incremental weighted average formulation
        *model = &*model * (1.0 - weight) + &sub.matrix * weight;

        true
    }
}
