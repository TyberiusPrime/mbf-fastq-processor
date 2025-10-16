---
weight: 2
title: TOML format

---
# TOML file format

mbf-fastq-processor pipelines are defined in a single TOML document. The format favours explicitness: every field is named, order is preserved, and unknown keys are rejected with a descriptive error.

## Canonical template

The repository maintains an [authoritative configuration scaffold](template.toml) (the same
content emitted by `mbf-fastq-processor template`). 

The contents are included [below](#maximal-example-template) for reference easy consumption in an LLM.

## Structure overview

| Section         | Required | Purpose                                                  |
|-----------------|----------|----------------------------------------------------------|
| `[input]`       | Yes      | Declare the FastQ segments and associated source files   |
| `[output]`      | Yes      | Configure how processed reads and reports are written    |
| `[[step]]`      | No*      | Define transformations, filters, tag operations, reports |
| `[options]`     | No       | Tune runtime knobs such as buffer sizes                  |
| `[barcodes.*]`  | Conditional | Supply barcode tables for demultiplexing            |

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
