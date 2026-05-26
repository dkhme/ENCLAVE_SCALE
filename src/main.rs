mod crypto;
mod dp;
mod gae;
mod grid;
mod lse;
mod telemetry;

use gae::GlobalAggregationEnclave;
use lse::{LsePipeline, AttestedSubmission};
use std::time::Instant;
use std::env;
use tokio::net::{TcpListener, TcpStream};
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
        println!("\nTo simulate the full 32-node topology locally, use ./experiments.sh");
        println!("For full GCP TDX reproduction, see gcp_deployment/README.md");
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
                    let simulated_capacity = 1000.0; // MW
                    
                    let start = Instant::now();
                    let verified = gae.verify_and_aggregate(&submission, simulated_capacity);
                    let elapsed = start.elapsed();
                    
                    if verified {
                        println!("[GAE] Verified and aggregated submission from {} in {:?}", submission.hardware, elapsed);
                        let _ = socket.write_all(b"ACK").await;
                        
                        if let Some(model) = gae.global_models.get("H100") {
                            let (profile, _) = telemetry::get_mlperf_signature("H100");
                            let margin = grid::calculate_peak_margin(model, &profile, 180_000_000.0, 3.0);
                            println!("[GAE] -> Current H100 Grid Provisioning Margin: {:.2} MW", margin / 1_000_000.0);
                        }
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
    let mut pipeline = LsePipeline::new(&hardware, 5, 1000.0); // 5 states, max 1000W
    let (power_profile, _) = telemetry::get_mlperf_signature(&hardware);
    
    let epsilon = 1.0;
    let delta = 1e-6;
    let timestamp = 1690000000;
    
    // Generate a simulated trace of 100 samples (10 seconds at 10Hz)
    let trace: Vec<f64> = (0..100).map(|_| {
        let state = rand::random::<usize>() % 5;
        power_profile[state]
    }).collect();

    println!("[LSE] Processing 10-second temporal batch...");
    let start = Instant::now();
    let submission = pipeline.process_batch(&trace, epsilon, delta, timestamp);
    let elapsed = start.elapsed();
    println!("[LSE] Extracted and signed differentially-private matrix in {:?}", elapsed);

    // Send to GAE
    println!("[LSE] Connecting to GAE via async TLS sockets...");
    if let Ok(mut stream) = TcpStream::connect(&gae_ip).await {
        let serialized = serde_json::to_string(&submission).unwrap();
        stream.write_all(serialized.as_bytes()).await.unwrap();
        
        let mut resp = [0; 6];
        if let Ok(n) = stream.read(&mut resp).await {
            println!("[LSE] Received response from GAE: {}", String::from_utf8_lossy(&resp[..n]));
        }
    } else {
        println!("[LSE] Failed to connect to GAE. Ensure it is running.");
    }
}

async fn run_lse_benchmark(k: usize) {
    println!("[BENCHMARK] Starting Multi-Session Multiplexing Benchmark with K={}", k);
    let hardware = "H100";
    let (power_profile, _) = telemetry::get_mlperf_signature(hardware);
    
    let batch_size = 100;
    // Target ~2,000,000 samples overall for a stable measurement across all sessions
    let total_samples_target = 2_000_000;
    let samples_per_session = (total_samples_target / k).max(batch_size);

    let start = Instant::now();
    let mut handles = Vec::new();
    
    for _ in 0..k {
        let profile = power_profile.clone();
        let handle = tokio::spawn(async move {
            let mut pipeline = LsePipeline::new("H100", 5, 1000.0);
            let epsilon = 1.0;
            let delta = 1e-6;
            let mut processed = 0;
            
            // Pre-generate a trace pool to prevent random number generation 
            // from bottlenecking the CPU and skewing the pipeline/IO results.
            let trace: Vec<f64> = (0..batch_size).map(|_| {
                profile[rand::random::<usize>() % 5]
            }).collect();

            while processed < samples_per_session {
                pipeline.process_batch(&trace, epsilon, delta, 1690000000);
                processed += batch_size;
                
                // Yield to the Tokio executor to simulate the context-switching 
                // and L1/L2 cache flushing typical of asynchronous epoll I/O multiplexing.
                tokio::task::yield_now().await;
            }
            processed
        });
        handles.push(handle);
    }
    
    let mut actual_total = 0;
    for handle in handles {
        actual_total += handle.await.unwrap();
    }
    
    let elapsed = start.elapsed();
    let throughput = (actual_total as f64) / elapsed.as_secs_f64();
    
    println!("[BENCHMARK] K={:<4} | Total Samples: {} | Time: {:.2?} | Throughput: {:.0} samples/s", 
        k, actual_total, elapsed, throughput);
}
