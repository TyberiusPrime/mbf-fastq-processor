# Segments

Modern sequencers, particularly Illumina sequencers, can read multiple times from one (amplified) DNA molecule, producing multiple 'segments' that together form a 'molecule' or 'fragment'.

## Definition and Configuration

Segments are defined in the `[input]` section of your TOML configuration. Each segment corresponds to one FASTQ file (or stream in interleaved formats), and segment names are arbitrary but should be meaningful.

```toml
[input]
read1 = ["sample_R1.fq.gz"]
read2 = ["sample_R2.fq.gz"]
index1 = ["sample_I1.fq.gz"]
```

In this example, three segments are defined: `read1`, `read2`, and `index1`.

## Common Segment Naming Conventions

While segment names are user-defined, certain conventions align with sequencing technologies:

### Paired-End Sequencing
```toml
[input]
read1 = ["lib_R1.fq.gz"]    # Forward read
read2 = ["lib_R2.fq.gz"]    # Reverse read
```

Common in RNA-seq, ChIP-seq, and genomic sequencing. Both reads sequence opposite ends of the same DNA fragment.

### Single-End Sequencing
```toml
[input]
read1 = ["lib.fq.gz"]       # Single read
```

Simpler, less expensive, but provides less information than paired-end.

### Indexed Libraries (Multiplexed Samples)
```toml
[input]
read1 = ["run_R1.fq.gz"]
read2 = ["run_R2.fq.gz"]
index1 = ["run_I1.fq.gz"]   # i7 index (first barcode)
index2 = ["run_I2.fq.gz"]   # i5 index (second barcode)
```

Index reads contain sample barcodes for demultiplexing multiple samples from a single sequencing run. Dual indexing reduces barcode collisions and enables higher multiplexing.

### Custom Naming
You can use any naming scheme that suits your workflow:
```toml
[input]
fwd = ["lib_F.fq.gz"]
rev = ["lib_R.fq.gz"]
umi = ["lib_UMI.fq.gz"]     # Unique Molecular Identifier read
```

## Segment Synchronization

**Critical:** All segments must contain the same number of reads, in the same order. The processor validates this during execution.

When a [step]({{< relref "docs/concepts/step.md" >}}) filters a fragment, **all segments** for that fragment are removed together, maintaining synchronization:

```toml
[[step]]
name = "FilterByLength"
segment = "read1"
minimum = 50
```

This removes fragments where `read1` is shorter than 50 bp, but it also removes the corresponding `read2`, `index1`, and `index2` segments for those fragments.

## Segment Targeting in Steps

Many steps operate on specific segments via the `segment` parameter:

```toml
[[step]]
name = "CutStart"
segment = "read1"     # Only trim read1
length = 10

[[step]]
name = "ValidateSeq"
segment = "index1"    # Only validate index1 sequences
```

### The "All" Pseudo-Segment

Some steps support `segment = "All"` to operate across all defined segments:

```toml
[[step]]
name = "FilterByLength"
segment = "All"       # Check all segments
minimum = 50          # All segments must be >= 50 bp
```

When using `"All"`, the step evaluates criteria across every segment and operates on the entire fragment.

## Segments vs Sources

When a step accepts a [`source`]({{< relref "docs/concepts/source.md" >}}) parameter instead of `segment`, it can read from:
- Segments (e.g., `"read1"`)
- Segment names (e.g., `"name:read1"`)
- Tag values (e.g., `"tag:barcode"`)

This provides greater flexibility for complex workflows involving metadata.

## Interleaved Segments

Interleaved FASTQ files combine multiple segments into a single file, alternating records:

```toml
[input]
source = ["interleaved.fq.gz"]
interleaved = ["read1", "read2"]
```

This declares two segments (`read1` and `read2`) from one file, where records alternate: fragment 1 read1, fragment 1 read2, fragment 2 read1, fragment 2 read2, etc.

## Performance Considerations

- **More segments = more I/O**: Each segment requires separate file reading/writing
- **Compression matters**: Gzip is slower than zstd but more portable
- **Parallel reading**: The processor reads input files in parallel when possible
- **Segment count affects memory**: More segments increase per-fragment memory footprint

## Practical Examples

### Adapter Trimming (Paired-End)
```toml
[[step]]
name = "ExtractIUPAC"
segment = "read1"
pattern = "AGATCGGAAGAGC"     # Illumina adapter
out_label = "adapter_r1"

[[step]]
name = "TrimAtTag"
segment = "read1"
in_label = "adapter_r1"
```

### Demultiplexing by Index
```toml
[[step]]
name = "ExtractIUPAC"
segment = "index1"
pattern = "ATCACG"            # Sample barcode
out_label = "barcode_match"

[[step]]
name = "FilterByTag"
in_label = "barcode_match"
action = "Keep"
```

### Quality Control Across Segments
```toml
[[step]]
name = "CalcMeanQuality"
segment = "read1"
out_label = "q_r1"

[[step]]
name = "CalcMeanQuality"
segment = "read2"
out_label = "q_r2"

[[step]]
name = "EvalExpression"
expression = "(q_r1 + q_r2) / 2"
out_label = "mean_q"

[[step]]
name = "FilterByNumericTag"
in_label = "mean_q"
minimum = 30
```

## See Also

- [Input section reference]({{< relref "docs/reference/input-section.md" >}}) for detailed configuration syntax
- [Source concept]({{< relref "docs/concepts/source.md" >}}) for understanding source parameters
- [Step concept]({{< relref "docs/concepts/step.md" >}}) for how steps interact with segments
