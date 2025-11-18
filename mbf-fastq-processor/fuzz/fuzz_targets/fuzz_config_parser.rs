#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(s) = std::str::from_utf8(data) {
        // Try to parse as TOML config
        // We use from_str which is the same path as the main application
        let _ = eserde::toml::from_str::<mbf_fastq_processor::Config>(s);

        // The fuzzer doesn't care if it fails - we're looking for panics,
        // crashes, or hangs, not parsing errors which are expected
    }
});
