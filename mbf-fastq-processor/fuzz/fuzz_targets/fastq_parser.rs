#![no_main]

use libfuzzer_sys::fuzz_target;
use mbf_fastq_processor::io::parsers::{FastqParser, Parser};
use std::io::Cursor;

/// Represents a parsed read with owned data for comparison
struct OwnedRead {
    name: Vec<u8>,
    seq: Vec<u8>,
    qual: Vec<u8>,
}

/// Serialize reads back to FASTQ format (Unix line endings)
fn reads_to_fastq(reads: &[OwnedRead]) -> Vec<u8> {
    let mut out = Vec::new();
    for read in reads {
        out.push(b'@');
        out.extend_from_slice(&read.name);
        out.push(b'\n');
        out.extend_from_slice(&read.seq);
        out.push(b'\n');
        out.push(b'+');
        out.push(b'\n');
        out.extend_from_slice(&read.qual);
        out.push(b'\n');
    }
    out
}

/// Parse all reads from a byte slice, returns None if parsing fails
fn parse_all(data: &[u8], buf_size: usize, target_reads: usize) -> Option<Vec<OwnedRead>> {
    let cursor = Cursor::new(data.to_vec());
    let mut parser = FastqParser::from_reader(Box::new(cursor), target_reads, buf_size);

    let mut reads = Vec::new();

    loop {
        let result = parser.parse();
        match result {
            Ok(pr) => {
                for read in &pr.fastq_block.entries {
                    reads.push(OwnedRead {
                        name: read.name.get(&pr.fastq_block.block).to_vec(),
                        seq: read.seq.get(&pr.fastq_block.block).to_vec(),
                        qual: read.qual.get(&pr.fastq_block.block).to_vec(),
                    });
                }
                if pr.was_final {
                    break;
                }
            }
            Err(_) => return None,
        }
    }

    Some(reads)
}

fuzz_target!(|data: &[u8]| {
    // Need at least 2 bytes for configuration
    if data.len() < 2 {
        return;
    }

    // Use first byte to derive buffer size: 4-67 bytes (small to stress boundaries)
    let buf_size = (data[0] % 64) as usize + 4;

    // Use second byte to derive target reads per block: 1-4
    let target_reads = (data[1] % 4) as usize + 1;

    // Remaining bytes are the FASTQ data
    let fastq_data = &data[2..];

    // First parse attempt
    let reads1 = match parse_all(fastq_data, buf_size, target_reads) {
        Some(r) => r,
        None => return, // Parse failed, that's fine for malformed input
    };

    // If we got no reads, nothing to round-trip
    if reads1.is_empty() {
        return;
    }

    // Serialize back to FASTQ
    let reconstructed = reads_to_fastq(&reads1);

    // Parse the reconstructed FASTQ (use same buffer params for consistency)
    let reads2 = parse_all(&reconstructed, buf_size, target_reads)
        .expect("Reconstructed FASTQ should always parse successfully");

    // Verify round-trip: reads should match
    assert_eq!(
        reads1.len(),
        reads2.len(),
        "Read count mismatch after round-trip"
    );

    for (i, (r1, r2)) in reads1.iter().zip(reads2.iter()).enumerate() {
        assert_eq!(r1.name, r2.name, "Name mismatch at read {}", i);
        assert_eq!(r1.seq, r2.seq, "Sequence mismatch at read {}", i);
        assert_eq!(r1.qual, r2.qual, "Quality mismatch at read {}", i);
    }
});
