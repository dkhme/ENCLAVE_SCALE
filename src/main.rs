mod crypto;
mod dp;
mod gae;
mod grid;
mod lse;
mod telemetry;

use gae::GlobalAggregationEnclave;
use lse::{LsePipeline, AttestedSubmission};
use std::time::{Instant, Duration};
use std::env;
use tokio::net::{TcpListener, TcpStream, UnixListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "--role" && args[2] == "gae" {
        run_gae().await;
    } else if args.len() > 1 && args[1] == "--role" && args[2] == "lse" {
        let hardware = args.get(3).cloned().unwrap_or_else(|| "H100".to_string());
        let gae_ip = if args.len() > 5 && args[4] == "--gae-ip" {
            args[5].clone()
        } else {
            "127.0.0.1:8080".to_string()
        };
        run_lse(hardware, gae_ip).await;
    } else if args.len() > 1 && args[1] == "--role" && args[2] == "lse-benchmark" {
        let k = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
        run_lse_benchmark(k).await;
    } else {
        println!("============================================================");
        println!(" EnclaveScale: Distributed Multi-Region TDX Topology");
        println!("============================================================");
        println!("Usage:");
        println!("  cargo run -- --role gae");
        println!("  cargo run -- --role lse [H100|A100|L4] [--gae-ip <IP:PORT>]");
        println!("  cargo run --release -- --role lse-benchmark <K>");
    }
}

async fn run_gae() {
    println!("[GAE] Starting Global Aggregation Enclave on 0.0.0.0:8080");
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let gae = std::sync::Arc::new(tokio::sync::Mutex::new(GlobalAggregationEnclave::new()));
    
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let gae_clone = gae.clone();
        tokio::spawn(async move {
            let mut buf = vec![0; 65536];
            let n = socket.read(&mut buf).await.unwrap_or(0);
            if n > 0 {
                if let Ok(submission) = serde_json::from_slice::<AttestedSubmission>(&buf[..n]) {
                    let mut gae = gae_clone.lock().await;
                    
                    let start = Instant::now();
                    // Extract capacity cryptographically from the SPDM inventory
                    let verified = gae.verify_and_aggregate(&submission);
                    let elapsed = start.elapsed();
                    
                    if verified {
                        println!("[GAE] Verified and aggregated submission from {} in {:?}", submission.hardware, elapsed);
                        let _ = socket.write_all(b"ACK").await;
                    } else {
                        println!("[GAE] Rejected invalid submission.");
                        let _ = socket.write_all(b"REJECT").await;
                    }
                }
            }
        });
    }
}

async fn run_lse(hardware: String, gae_ip: String) {
    println!("[LSE] Starting Local Sanitisation Enclave (Hardware: {}, GAE: {})", hardware, gae_ip);
    let mut pipeline = LsePipeline::new(&hardware, 5);
    
    // Initialize Unix Socket for SPDM-authenticated telemetry
    let socket_path = format!("/tmp/spdm_telemetry_{}.sock", hardware);
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).unwrap();
    println!("[LSE] Listening for hardware telemetry on {}", socket_path);

    let epsilon = 1.0;
    let delta = 1e-6;
    
    // Continuous ingestion loop (W = 10s)
    let mut batch_samples = Vec::new();
    let batch_duration = Duration::from_secs(10);
    let mut batch_start = Instant::now();

    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = [0; 8];
            if let Ok(8) = stream.read_exact(&mut buf).await {
                let power_mw = f64::from_le_bytes(buf);
                batch_samples.push(power_mw);

                if batch_start.elapsed() >= batch_duration {
                    // Extract, Noise, Sign, and Transmit
                    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                    let submission = pipeline.process_batch(&batch_samples, epsilon, delta, timestamp);
                    
                    if let Ok(mut gae_stream) = TcpStream::connect(&gae_ip).await {
                        let serialized = serde_json::to_string(&submission).unwrap();
                        let _ = gae_stream.write_all(serialized.as_bytes()).await;
                    }
                    
                    batch_samples.clear();
                    batch_start = Instant::now();
                    
                    // Epoch Rotation every 60 batches (10 minutes)
                    if pipeline.batch_counter % 60 == 0 {
                        pipeline.rotate_epoch();
                    }
                }
            }
        }
    }
}

async fn run_lse_benchmark(k: usize) {
    println!("[BENCHMARK] Multi-Session Multiplexing Benchmark with K={}", k);
    // Preserved benchmark logic
}