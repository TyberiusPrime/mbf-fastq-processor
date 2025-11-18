// Example: Seeking in gzip files
//
// This example demonstrates the seeking capability of rapidgzip,
// which is not available with standard gzip readers.
//
// Usage: cargo run --example seek_example <input.gz>

use rapidgzip_wrapper::ParallelGzipReader;
use std::env;
use std::io::{Read, Seek, SeekFrom};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.gz>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];

    println!("Opening {}...", input_path);
    let mut reader = ParallelGzipReader::open(input_path, 0)?;

    // Read first 100 bytes
    println!("\n--- First 100 bytes ---");
    let mut buffer = vec![0u8; 100];
    let n = reader.read(&mut buffer)?;
    println!("{}", String::from_utf8_lossy(&buffer[..n]));

    // Seek to position 1000
    println!("\n--- Seeking to position 1000 ---");
    let new_pos = reader.seek(SeekFrom::Start(1000))?;
    println!("New position: {}", new_pos);

    // Read another 100 bytes
    let n = reader.read(&mut buffer)?;
    println!("{}", String::from_utf8_lossy(&buffer[..n]));

    // Seek to 500 bytes before the end
    println!("\n--- Last 500 bytes (seeking from end) ---");
    reader.seek(SeekFrom::End(-500))?;
    let mut end_buffer = Vec::new();
    reader.read_to_end(&mut end_buffer)?;
    println!("{}", String::from_utf8_lossy(&end_buffer));

    // Seek back to the beginning
    println!("\n--- Back to start ---");
    reader.seek(SeekFrom::Start(0))?;
    let n = reader.read(&mut buffer)?;
    println!("First {} bytes again:", n);
    println!("{}", String::from_utf8_lossy(&buffer[..n]));

    Ok(())
}
