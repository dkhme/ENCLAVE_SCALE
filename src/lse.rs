use nalgebra::DMatrix;
use crate::telemetry::Discretiser;
use crate::dp::apply_gaussian_noise;
use crate::crypto::{LseIdentity, hash_payload};
use ed25519_dalek::Signer;

pub struct LsePipeline {
    pub identity: LseIdentity,
    pub hardware_type: String,
    pub batch_counter: u64,
    discretiser: Discretiser,
    num_states: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AttestedSubmission {
    pub matrix: DMatrix<f64>,
    pub q_init: String,
    pub vk: ed25519_dalek::VerifyingKey,
    pub signature: ed25519_dalek::Signature,
    pub hardware: String,
    pub timestamp: u64,
    pub batch_counter: u64,
}

impl LsePipeline {
    pub fn new(hardware_type: &str, num_states: usize, max_power: f64) -> Self {
        Self {
            identity: LseIdentity::new(),
            hardware_type: hardware_type.to_string(),
            batch_counter: 0, // Strictly monotonic batch counter (Anti-Replay)
            discretiser: Discretiser::new(num_states, max_power),
            num_states,
        }
    }

    /// Implements Algorithm 1: LSE State Extraction and DP Pipeline
    pub fn process_batch(&mut self, traces: &[f64], epsilon: f64, delta: f64, timestamp: u64) -> AttestedSubmission {
        self.batch_counter += 1;
        let mut m = DMatrix::zeros(self.num_states, self.num_states);
        
        // Step 1: Plaintext State Extraction
        let mut prev_state = self.discretiser.process(traces[0]);
        for &power in traces.iter().skip(1) {
            let curr_state = self.discretiser.process(power);
            m[(prev_state, curr_state)] += 1.0;
            prev_state = curr_state;
        }

        // 2. DP Noise Injection (calibrated to Delta_2 f = sqrt(6))
        apply_gaussian_noise(&mut m, epsilon, delta);

        // 3. Remote Attestation Cryptographic Binding (Amortised)
        let payload_hash = hash_payload(&m, &self.hardware_type, timestamp, self.batch_counter);
        let signature = self.identity.signing_key.sign(&payload_hash);

        AttestedSubmission {
            matrix: m,
            q_init: self.identity.q_init.clone(),
            vk: self.identity.verifying_key,
            signature,
            hardware: self.hardware_type.clone(),
            timestamp,
            batch_counter: self.batch_counter,
        }
    }
}
