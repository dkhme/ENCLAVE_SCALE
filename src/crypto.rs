use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use nalgebra::DMatrix;

/// Simulates the hardware-bound identity of a TDX Local Sanitisation Enclave (Section 4.3)
pub struct LseIdentity {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub q_init: String, // Simulated DCAP quote binding the VK to the MRTD
}

impl LseIdentity {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        // Note: To preserve cross-platform build compatibility for artifact evaluation, 
        // this routine simulates the DCAP quoting enclave interface. In a production 
        // deployment, this relies on `tdx_attest_get_quote` to bind the VK to the MRTD.
        let q_init = format!("TDX_DCAP_QUOTE_BINDING_{}", hex::encode(verifying_key.as_bytes()));
        
        Self { signing_key, verifying_key, q_init }
    }
}

/// Computes H = SHA256( M_i || h || timestamp || b ) as defined in Algorithm 1
pub fn hash_payload(
    matrix: &DMatrix<f64>,
    hardware: &str,
    timestamp: u64,
    batch_counter: u64,
) -> Vec<u8> {
    let mut hasher = Sha256::new();
    
    // Serialize matrix deterministically
    for val in matrix.iter() {
        hasher.update(val.to_le_bytes());
    }
    
    hasher.update(hardware.as_bytes());
    hasher.update(timestamp.to_le_bytes());
    hasher.update(batch_counter.to_le_bytes());
    
    hasher.finalize().to_vec()
}
