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
    pub q_init: Vec<u8>,
    pub vk: ed25519_dalek::VerifyingKey,
    pub signature: ed25519_dalek::Signature,
    pub hardware: String,
    pub timestamp: u64,
    pub batch_counter: u64,
}

impl LsePipeline {
    pub fn new(hardware_type: &str, num_states: usize) -> Self {
        Self {
            identity: LseIdentity::new(hardware_type),
            hardware_type: hardware_type.to_string(),
            batch_counter: 0,
            discretiser: Discretiser::new(hardware_type),
            num_states,
        }
    }
    
    pub fn rotate_epoch(&mut self) {
        // Rotate identities to bound privacy loss
        self.identity = LseIdentity::new(&self.hardware_type);
        self.batch_counter = 0;
    }

    /// Implements Algorithm 1: LSE State Extraction retaining self-loops.
    pub fn process_batch(&mut self, traces: &[f64], epsilon: f64, delta: f64, timestamp: u64) -> AttestedSubmission {
        self.batch_counter += 1;
        let mut m = DMatrix::zeros(self.num_states, self.num_states);
        
        if traces.is_empty() {
            // Empty batch fallback
        } else {
            // Extraction pipeline explicitly retains sequential identical samples (self-loops).
            let mut prev_state = self.discretiser.process(traces[0]);
            for &power in traces.iter().skip(1) {
                let curr_state = self.discretiser.process(power);
                m[(prev_state, curr_state)] += 1.0;
                prev_state = curr_state;
            }
        }

        // DP Noise Injection
        apply_gaussian_noise(&mut m, epsilon, delta);

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