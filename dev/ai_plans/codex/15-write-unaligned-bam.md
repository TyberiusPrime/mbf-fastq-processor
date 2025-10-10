# Plan: Write Unaligned BAM Support

1. Re-read issue 0041 and audit existing output/file-format code to see how FASTQ writers are set up, what config validation enforces, and where FileFormat is matched.
2. Extend config enums/validation and the output writer infrastructure so that `format = "bam"` becomes a valid choice, producing an appropriate writer instance and required metadata/header state.
3. Implement unaligned BAM emission for both segmented and interleaved outputs, ensuring flags, mate metadata, and hashing/finishing behaviour stay correct.
4. Document the new BAM format option (including any hashing/stdout limitations) and add targeted automated tests that exercise both single-segment and paired/interleaved BAM outputs.
5. Run formatting and the relevant cargo test suite, inspect results, and capture anything noteworthy for the final report.
