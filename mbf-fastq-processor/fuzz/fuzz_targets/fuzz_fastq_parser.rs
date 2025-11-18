#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    // Create a temporary file with the fuzz input
    if let Ok(mut tmpfile) = tempfile::NamedTempFile::new() {
        // Write the fuzzer data to the temp file
        let _ = tmpfile.write_all(data);
        let _ = tmpfile.flush();

        // Try to parse it as a FastQ file
        // We use apply_to_read_sequences which is a public API
        let _ = mbf_fastq_processor::io::apply_to_read_sequences(
            tmpfile.path(),
            &mut |_seq| {
                // Just consume the sequences, we're testing for crashes/panics
            },
            None,
        );
    }
});
