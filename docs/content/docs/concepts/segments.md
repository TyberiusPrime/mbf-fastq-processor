# Segments

Modern sequencers, particularly Illumina sequencers, can read multiple times from one (amplified) DNA molecule, producing multiple 'segments' (often called 'reads') that together form a 'molecule' or 'fragment'.

## Definition and Configuration

Segments are defined in the `[input]` section of your TOML configuration. Each segment corresponds to one FASTQ file (or stream in interleaved formats), and segment names are arbitrary but should be meaningful.

```toml # ignore_in_test
[input]
    read1 = ["sample_R1.fq.gz"]
    read2 = ["sample_R2.fq.gz"]
    index1 = ["sample_I1.fq.gz"]
```

In this example, three segments are defined: `read1`, `read2`, and `index1`.

## Common Segment Naming Conventions

While segment names are user-defined, certain conventions align with sequencing technologies:

### Paired-End Sequencing
```toml # ignore_in_test
[input]
    read1 = ["lib_R1.fq.gz"]    # Forward read
    read2 = ["lib_R2.fq.gz"]    # Reverse read
```

Common in RNA-seq. Often the reads are on opposing strands.

### Single-End Sequencing
```toml # ignore_in_test
[input]
    read1 = ["lib.fq.gz"]       # Single read
```

Common in ChIP-seq or targeted sequencing.

### Indexed Libraries (Multiplexed Samples)
```toml # ignore_in_test
[input]
        read1 = ["run_R1.fq.gz"]
        read2 = ["run_R2.fq.gz"]
        index1 = ["run_I1.fq.gz"]   # i7 index (first barcode)
        index2 = ["run_I2.fq.gz"]   # i5 index (second barcode)
```

Index reads contain sample barcodes for demultiplexing multiple samples from a single sequencing run. Dual indexing reduces barcode collisions and enables higher multiplexing.

### Custom Naming
You can use any naming scheme that suits your workflow.

Note that these end up in the output file names as well.

```toml # ignore_in_test
[input]
    fwd = ["lib_F.fq.gz"]
    rev = ["lib_R.fq.gz"]
    umi = ["lib_UMI.fq.gz"]     # Unique Molecular Identifier read
```

## Segment Synchronization

**Critical:** All segments must contain the same number of reads, in the same order. The processor validates this during execution by [spot checking the read names]({{< relref "docs/reference/validation-steps/ValidateReadPairing.md" >}}).

When a [step]({{< relref "docs/concepts/step.md" >}}) filters a molecule, **all segments** for that fragment are removed together, maintaining synchronization.

## Segment Targeting in Steps

Many steps operate on specific segments via the `segment` parameter:

```toml
[[step]]
    action = "CutStart"
    segment = "read1"     # trim read1
    n = 10

[[step]]
    action = "ValidateSeq"
    segment = "index1"    # Only validate index1 sequences
```

### The "All" Pseudo-Segment

Some steps support `segment = "All"` to operate across all defined segments:

```toml
[[step]]
    action = "CalcLength"
    segment = "All"       # Check all segments
    out_label = "sum_len"
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

```toml # ignore_in_test
[input]
    source = ["interleaved.fq.gz"]
    interleaved = ["read1", "read2"]
```

This declares two segments (`read1` and `read2`) from one file, where records alternate: fragment 1 read1, fragment 1 read2, fragment 2 read1, fragment 2 read2, etc.

## See Also

- [Input section reference]({{< relref "docs/reference/input-section.md" >}}) for detailed configuration syntax
- [Source concept]({{< relref "docs/concepts/source.md" >}}) for understanding source parameters
- [Step concept]({{< relref "docs/concepts/step.md" >}}) for how steps interact with segments