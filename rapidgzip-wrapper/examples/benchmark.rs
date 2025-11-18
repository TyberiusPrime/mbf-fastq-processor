// Example: Benchmark parallel decompression
//
// This example demonstrates the performance benefits of parallel decompression
// by comparing single-threaded vs multi-threaded decompression.
//
// Usage: cargo run --release --example benchmark <input.gz>

use rapidgzip_wrapper::ParallelGzipReader;
use std::env;
use std::io::Read;
use std::time::Instant;

fn decompress_file(path: &str, num_threads: usize) -> anyhow::Result<(u64, u128)> {
    let start = Instant::now();

    let mut reader = ParallelGzipReader::open(path, num_threads)?;
    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
    let mut total_bytes = 0u64;

    loop {
        match reader.read(&mut buffer)? {
            0 => break,
            n => total_bytes += n as u64,
        }
    }

    let elapsed = start.elapsed().as_millis();

    Ok((total_bytes, elapsed))
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.gz>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];

    println!("Benchmarking decompression of: {}\n", input_path);

    // Get number of CPUs
    let num_cpus = num_cpus::get();
    println!("Detected {} CPU cores\n", num_cpus);

    // Test with different thread counts
    let thread_counts = vec![1, 2, 4, num_cpus, 0]; // 0 = auto

    let mut results = Vec::new();

    for &threads in &thread_counts {
        let thread_label = if threads == 0 {
            "auto".to_string()
        } else {
            threads.to_string()
        };

        print!("Testing with {} threads... ", thread_label);
        std::io::Write::flush(&mut std::io::stdout())?;

        let (bytes, time_ms) = decompress_file(input_path, threads)?;

        let throughput_mbps = (bytes as f64) / (time_ms as f64) / 1000.0;

        println!("Done!");
        println!(
            "  Time: {}ms, Throughput: {:.2} MB/s",
            time_ms, throughput_mbps
        );

        results.push((thread_label, bytes, time_ms, throughput_mbps));
    }

    // Print summary
    println!("\n--- Summary ---");
    println!(
        "{:<10} {:>15} {:>15} {:>15}",
        "Threads", "Bytes", "Time (ms)", "Throughput (MB/s)"
    );
    println!("{:-<55}", "");

    for (threads, bytes, time, throughput) in &results {
        println!(
            "{:<10} {:>15} {:>15} {:>15.2}",
            threads, bytes, time, throughput
        );
    }

    // Calculate speedup
    if let Some((_, _, single_thread_time, _)) = results.first() {
        println!("\n--- Speedup vs Single Thread ---");
        for (threads, _, time, _) in results.iter().skip(1) {
            let speedup = *single_thread_time as f64 / *time as f64;
            println!("{:<10} {:.2}x", threads, speedup);
        }
    }

    Ok(())
}

// Note: This example requires the num_cpus crate
// Add to Cargo.toml:
// [dev-dependencies]
// num_cpus = "1.16"
