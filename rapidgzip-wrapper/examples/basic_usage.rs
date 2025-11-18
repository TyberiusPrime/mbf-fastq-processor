// Example: Basic usage of rapidgzip-wrapper
//
// This example demonstrates how to use the ParallelGzipReader to decompress
// a gzip file using multiple threads.
//
// Usage: cargo run --example basic_usage <input.gz>

use rapidgzip_wrapper::ParallelGzipReader;
use std::env;
use std::io::{self, Read, Write};

fn main() -> anyhow::Result<()> {
    // Get input file from command line
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.gz>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];

    // Open the gzip file with automatic thread detection (0 = auto)
    println!("Opening {}...", input_path);
    let mut reader = ParallelGzipReader::open(input_path, 0)?;

    // Get the decompressed size if available
    if let Some(size) = reader.size()? {
        println!("Decompressed size: {} bytes", size);
    } else {
        println!("Decompressed size: unknown (will be determined during reading)");
    }

    // Read and decompress the file
    println!("Decompressing...");
    let mut buffer = [0u8; 8192];
    let mut total_bytes = 0u64;
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    loop {
        match reader.read(&mut buffer)? {
            0 => break, // EOF
            n => {
                handle.write_all(&buffer[..n])?;
                total_bytes += n as u64;
            }
        }
    }

    eprintln!("\nDecompressed {} bytes", total_bytes);

    Ok(())
}
