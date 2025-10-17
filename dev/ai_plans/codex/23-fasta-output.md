# Plan: FASTA output support

1. Refactor configuration enums: introduce `FileFormat` (fastq/fasta/bam/none) and `CompressionFormat` (uncompressed/gzip/zstd); keep suffix logic and legacy config compatibility; validate new combinations (e.g. compression with BAM).
2. Update output infrastructure: adjust `HashedAndCompressedWriter`, `OutputFile`, and related plumbing to take compression separately and add a dedicated FASTA variant in `OutputFileKind`.
3. Implement FASTA serialization path: add `WrappedFastQRead::as_fasta`, use it in standard and interleaved block writers, and ensure demultiplexed writes respect the new branch.
4. Touch dependent modules (e.g. StoreTagInFastQ, report helpers) so they compile with the new enums and validation behaviour.
5. Format and run `cargo check` to confirm the changes.
