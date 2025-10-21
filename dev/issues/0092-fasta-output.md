status: closed

# fasta output

The goal is to introduce Fasta output and
adjust the existing 'file_format' which conflates compression format
and read format, to separate them.

Extend lib::OutputFileKind with Fasta<OutputWriter<'a>.

Replace the config.mod.rs::FileFormat with two enums,

one FileFormat(FastQ, Fasta, BAM)
and one CompressionFormat (Uncompressed, Gzip, zstd).

validate_compression_level_u8 takes the later.

The output struct now takes both. Setting CompressionFormat on BAM is invalid
(needs validation).

Extend lib.rs::OutputFile::new() to take both and create the right OutputFileKind.
Adjust the other methods accordingly to handle Fasta as well.

Introduce WrappedFastQRead.as_fasta analog to append_as_fastq.
Use it in output_block_inner and output_block_inner_interleaved.
