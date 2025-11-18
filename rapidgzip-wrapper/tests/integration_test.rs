use rapidgzip_wrapper::ParallelGzipReader;
use std::io::Read;

#[test]
fn test_decompress_gzip_file() {
    // Read the test gzip file
    let test_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test.txt.gz");

    let mut reader = ParallelGzipReader::open(test_file, 0)
        .expect("Failed to open test file");

    // Read the decompressed content
    let mut content = String::new();
    reader.read_to_string(&mut content)
        .expect("Failed to read from gz file");

    // Verify the content contains expected text
    assert!(content.contains("Hello from rapidgzip"));
    assert!(content.contains("quick brown fox"));
    assert!(content.contains("Line 2"));
}

#[test]
fn test_tell_and_seek() {
    let test_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test.txt.gz");

    let mut reader = ParallelGzipReader::open(test_file, 0)
        .expect("Failed to open test file");

    // Initially at position 0
    assert_eq!(reader.tell().unwrap(), 0);

    // Read some bytes
    let mut buf = [0u8; 10];
    reader.read(&mut buf).expect("Failed to read");

    // Position should have advanced
    assert_eq!(reader.tell().unwrap(), 10);

    // Seek back to start
    use std::io::Seek;
    reader.seek(std::io::SeekFrom::Start(0)).expect("Failed to seek");
    assert_eq!(reader.tell().unwrap(), 0);

    // Read again should give same bytes
    let mut buf2 = [0u8; 10];
    reader.read(&mut buf2).expect("Failed to read after seek");
    assert_eq!(buf, buf2);
}

#[test]
fn test_eof() {
    let test_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test.txt.gz");

    let mut reader = ParallelGzipReader::open(test_file, 0)
        .expect("Failed to open test file");

    // Not at EOF initially
    assert!(!reader.is_eof().unwrap());

    // Read all content
    let mut content = Vec::new();
    reader.read_to_end(&mut content).expect("Failed to read");

    // Now at EOF
    assert!(reader.is_eof().unwrap());
}

#[test]
fn test_crc32() {
    let test_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test.txt.gz");

    let mut reader = ParallelGzipReader::open(test_file, 0)
        .expect("Failed to open test file");

    // Enable CRC32 verification
    reader.set_crc32_enabled(true).expect("Failed to set CRC32");

    // Read should succeed with valid CRC
    let mut content = Vec::new();
    reader.read_to_end(&mut content).expect("CRC32 verification should pass");
}

#[test]
fn test_threaded_decompression() {
    let test_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test.txt.gz");

    // Try different thread counts
    for threads in [1, 2, 4, 0] {
        let mut reader = ParallelGzipReader::open(test_file, threads)
            .expect(&format!("Failed to open with {} threads", threads));

        let mut content = String::new();
        reader.read_to_string(&mut content)
            .expect("Failed to read");

        assert!(content.contains("Hello from rapidgzip"),
                "Content mismatch with {} threads", threads);
    }
}
