#![no_main]

use libfuzzer_sys::fuzz_target;

// This target fuzzes the custom deserializers in the config module
// These handle things like DNA sequences, IUPAC codes, barcodes, etc.

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(s) = std::str::from_utf8(data) {
        // Test TOML with various field types that use custom deserializers
        // We test various configurations that exercise different deserializers

        // Test input configuration with flexible string/array syntax
        let input_config = format!(
            r#"
[input]
read1 = {}
"#,
            s
        );
        let _ = eserde::toml::from_str::<mbf_fastq_processor::config::Input>(&input_config);

        // Test barcode parsing (IUPAC DNA validation)
        let barcode_config = format!(
            r#"
[barcodes]
barcode_to_name = {{ {} = "sample1" }}
"#,
            s
        );
        let _ = eserde::toml::from_str::<mbf_fastq_processor::config::Barcodes>(&barcode_config);

        // Test segment configuration
        if !s.is_empty() && !s.contains('"') && !s.contains('\'') {
            let segment_config = format!(
                r#"
[[step]]
Inspect.segment = "{}"
"#,
                s
            );
            let _ = eserde::toml::from_str::<Vec<mbf_fastq_processor::transformations::Transformation>>(&segment_config);
        }
    }
});
