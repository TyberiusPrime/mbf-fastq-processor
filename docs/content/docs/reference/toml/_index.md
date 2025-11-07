---
weight: 2
title: TOML format
---

# TOML file format

mbf-fastq-processor pipelines are defined in a single [TOML](https://toml.io/en/) 1.0 formatted document.
The format favours explicitness: every field is named, order is preserved, and unknown keys are rejected with a descriptive error.

## Canonical template

The repository maintains an [authoritative configuration scaffold](template.toml) (the same
content emitted by `mbf-fastq-processor template`).

The contents are included [below](#maximal-example-template) for reference easy consumption in an LLM.

## Structure overview

| Section           | Required    | Purpose                                                  |
| ----------------- | ----------- | -------------------------------------------------------- |
| `[input]`         | Yes         | Declare the FASTQ segments and associated source files   |
| `[input.options]` | Conditional | Configure format-specific input toggles (FASTA/BAM)      |
| `[output]`        | Yes         | Configure how processed reads and reports are written    |
| `[[step]]`        | No\*        | Define transformations, filters, tag operations, reports |
| `[options]`       | No          | Tune runtime knobs such as buffer sizes                  |
| `[barcodes.*]`    | Conditional | Supply barcode tables for demultiplexing                 |

`[[step]]` entries are optional in the technical sense—an empty pipeline simply copies data between input and output—but in practice most configurations contain at least one transformation or report.

## Minimal example

```toml
[input]
    read1 = "file_1.fq"
    read2 = ["file_2.fq"] # lists concatenate multiple files for a segment

[output]
    prefix = "processed"
    format = "Raw" # or Gzip/Zstd/Bam/None

[[step]]
    action = "CutStart"
    segment = "read1"
    n = 3
```

Key rules:

- Steps execute top-to-bottom exactly as written.
- Field names are case-insensitive when matching segments but consistent casing improves readability.
- Arrays of tables (`[[step]]`) must come after their shared configuration. Intervening scalar keys apply to the most recent table.

Refer to the [Input section]({{< relref "docs/reference/input-section.md" >}}) and [Output section]({{< relref "docs/reference/output-section.md" >}}) pages for exhaustive key listings, supported compression formats, and compatibility notes.

### Additional tables

Some steps require additional tables outside the main `[[step]]` list—for example [Demultiplex]({{< relref "docs/reference/Demultiplex.md" >}}) expects `[barcodes.<name>]` definitions. Place those tables anywhere in the file; they are parsed before execution begins, so forward and backward references are both valid.

### Comments and formatting

TOML supports `#` comments. Leverage them to annotate why a step exists or to document barcode provenance. The parser enforces strict validation: spelling mistakes such as `actionn = "CutStart"` will cause an immediate error instead of being silently ignored.

## Why TOML?

We deliberately avoided deep CLI flag hierarchies and configuration formats without comments. TOML offers ordered arrays for sequencing steps, nested tables for barcode definitions, and human-friendly syntax that is widely adopted in both Python and Rust ecosystems.

Curious about complex structures? The [Demultiplex reference]({{< relref "docs/reference/Demultiplex.md" >}}) showcases nested tables and arrays combined with the TOML array-of-tables syntax.

## Maximal example (template)

```toml
{{% include "template.toml" %}}
```

## Isn't this awfully verbose?

Configuration being understandable is much more important than being terse,
and that's what we strife for.

It is usually written (or is copy/pasted) with the documentation at hand, so typing is
not a limting factor..

Our anti-example are tools that end up being called like this
(no shade on fastp - bioinformatic tools are overwhelmingly like this):

```bash
fastp \
    --in1 r1.fastq.gz \
    --in2 r2.fastq.gz \
    -m \
    --merged_out merged.fastp.gz \
    --out1 read1.fastp.gz \
    --out2 read2.fastp.gz \
    -A -G -Q -L
```

Which is reasonably clear, until you get to the one-letter-options. In this case, they
turn on 'merge mode' ('-m', which you might have guessed) and disable some default processing steps ('-A -G -Q -L').

Here's the mbf-fastq-processor equivalent, which we think as being more maintainable:

```toml
[input]
    read1 = 'r1.fastq.gz'
    read2 = 'r2.fastq.gz'

[[step]]
    action = 'MergeReads'
	algorithm = "Fastp"
    min_overlap = 30
    max_mismatch_rate = 0.2
	max_mismatch_count = 5
    no_overlap_strategy = 'AsIs'
    reverse_complement_segment2 = true
    segment1 = 'read1'
    segment2 = 'read2'

[output]
	prefix = "output"
	compression = "gzip"
```

It also illustrates our stance on configuration defaults: Keep them minimal. You can never
change a default without unexpectedly and silently breaking some user's pipeline.

For example, if fastp added another default processing step in a future version, users of fastp would
have to add another 'disable' command line flag to their invocations to keep the same behaviour as before.

It's much better to make them write it down explicitly in the first place.

To make getting started easier, 
we allow a number of aliases,
spelling variations,
and do defaults when they're absolutely obvious 
- for example in [Swap]({{< relref "docs/reference/modification-steps/Swap.md" >}}), when there are only 2 segments defined 
- or when we really really don't expect them to change.
For example defaulting to ['outputting all segments']({{< relref "docs/reference/output-section.md" >}}), 
or ([spot checking read name pairs]({{< relref "docs/reference/validation-steps/SpotCheckReadPairing.md" >}}))
