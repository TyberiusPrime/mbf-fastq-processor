---
weight: 10
bookFlatSection: true
title: "Concepts"
---

# High level

mbf-fastq-processor ingests any number of FASTQ files, applies a user-defined sequence of steps, and emits transformed FASTQs and/or structured reports.

```
FASTQ segments  ('reads')
       ↓
[extract | modify | filter | report] 
       ↓ 
FASTQs / tables / HTML reports
````
Each step is explicit: there are no hidden defaults, and order matters. 

## Terminology

- **Fragment / molecule** – the logical sequencing record composed of one or more segments (e.g., read1 / read2 / index1 / index2). The piece of DNA the sequencer operated on.
- **Segment** – one 'read' from a fragment. Segment streams are named in the `[input]` section (commonly `read1`, `read2`, `index1`, etc.). Many steps operate on a specific segment.
- **Tag** – metadata derived from a fragment and stored under a label; later steps may consume, modify, or filter on it it.
- **Step** – an entry in the `[[step]]` array that mutates, filters, validates, or reports on fragments.

## Parameterisation

Pipelines live in a TOML document. Steps execute top-to-bottom, and you may repeat a step type any number of times (for example, collect a report both before and after filtering).

Values in the TOML file are explicit by design. Where defaults exist, they are documented in the [reference]({{< relref "docs/reference/_index.md" >}}).

## Input files

mbf-fastq-processor reads uncompressed, gzipped, or zstd-compressed FASTQ files ([and other file formats]({{< relref "docs/reference/input-section.md" >}}#file-formats)). Multiple files can be concatenated per segment. Every segment must supply the same number of reads to preserve fragment pairing.

Interleaved FASTQ files are also supported—declare a single source and enumerate segment names via `interleaved = [...]` (see the [input section reference]({{< relref "docs/reference/input-section.md" >}})).

FASTQs should comply with the format described by [Cock et al.](https://academic.oup.com/nar/article/38/6/1767/3112533). Data on the `+` line is ignored during parsing (and hence omitted from outputs).

## Output files

Output filenames derive from the configured prefix plus segment names (for example, `output_read1.fq.gz`). Interleaved outputs use `interleaved` as a segment name.

Reports use `prefix.html` / `prefix.json`. Additional artifacts such as checksums or per-barcode files are controlled by specific steps and `[options]` entries.

See the [output section reference]({{< relref "docs/reference/output-section.md" >}}) for supported formats and modifiers.

## Steps and Targets

Every step sees whole fragments so paired segments stay in lock-step: if you filter a fragment based on `read1`, the associated `read2` and any index reads disappear alongside it.

Many steps accept a `segment` argument to restrict their work to a specific input stream, while still retaining awareness of the whole fragment. Some steps also support a more flexible `source` parameter that can read from segment sequences, read names, or tag values.

Tag-generating steps must be paired with consumers. mbf-fastq-processor will error if a label is produced but never used, helping you catch typos early.

## Core Concepts

To build effective pipelines, understanding these core concepts is essential:

### [Steps]({{< relref "docs/concepts/step.md" >}})
Steps are the building blocks of your pipeline. They fall into five categories:
- **Modification**: Transform sequences, trim, merge, or restructure fragments
- **Filter**: Remove fragments based on quality, content, or statistical criteria
- **Tag**: Extract patterns, calculate metrics, or assign metadata labels
- **Report**: Generate statistics, visualizations, and quality reports
- **Validation**: Assert data correctness and catch formatting errors

Steps execute sequentially in the order specified, and the same step type can appear multiple times.

### [Segments]({{< relref "docs/concepts/segments.md" >}})
Segments represent the different reads from a single DNA fragment (e.g., `read1`, `read2`, `index1`). Modern sequencers can produce multiple segments per fragment, and mbf-fastq-processor keeps them synchronized throughout processing.

Key points:
- Segment names are user-defined in the `[input]` section
- All segments must contain the same number of reads
- When a fragment is filtered, all its segments are removed together
- Many steps target specific segments while maintaining fragment integrity

### [Tags]({{< relref "docs/concepts/tag.md" >}})
Tags store fragment-derived metadata under labeled names. They enable sophisticated workflows by decoupling data extraction from data usage.

Tag types:
- **Location+Sequence**: Regions within segments (e.g., adapter positions)
- **Sequence-only**: String values without position (e.g., extracted barcodes)
- **Numeric**: Calculated metrics (e.g., mean quality, GC content)
- **Boolean**: True/false flags (e.g., has_adapter, is_long_read)

Tags are created with `out_label` parameters, consumed with `in_label` parameters, and the processor validates that every tag is both defined and used.

### [Sources]({{< relref "docs/concepts/source.md" >}})
Some steps accept a `source` parameter instead of `segment`, allowing them to read from:
- Segment sequences: `"read1"`
- Segment names (FASTQ headers): `"name:read1"`
- Tag values: `"tag:barcode"`

This flexibility enables advanced patterns like extracting UMIs from read names, searching within previously extracted regions, or validating tag-derived sequences.

## Workflow Patterns

Common pipeline patterns include:

**Quality filtering:**
```toml
[[step]]
name = "CalcMeanQuality"
segment = "read1"
out_label = "mean_q"

[[step]]
name = "FilterByNumericTag"
in_label = "mean_q"
minimum = 30
```

**Adapter trimming:**
```toml
[[step]]
name = "ExtractIUPAC"
segment = "read1"
pattern = "AGATCGGAAGAGC"
out_label = "adapter"

[[step]]
name = "TrimAtTag"
segment = "read1"
in_label = "adapter"
trim_side = "RightOfMatch"
```

**Demultiplexing:**
```toml
[[step]]
name = "ExtractIUPAC"
segment = "index1"
pattern = "ATCACG"
out_label = "barcode"

[[step]]
name = "FilterByTag"
in_label = "barcode"
action = "Keep"
```

## Demultiplexing

Demultiplexing splits the fragment stream into multiple outputs based on barcodes, length thresholds, or any tag value. This enables processing of multiplexed sequencing runs where multiple samples share a single lane.

Demultiplexing can be based on:
- Index read sequences
- Inline barcodes within read sequences
- Read length ranges
- Any tag-derived criteria

## Further Reading

- **Detailed concept pages**: Explore [Step]({{< relref "docs/concepts/step.md" >}}), [Segments]({{< relref "docs/concepts/segments.md" >}}), [Tag]({{< relref "docs/concepts/tag.md" >}}), and [Source]({{< relref "docs/concepts/source.md" >}}) for in-depth explanations and examples
- **Reference documentation**: See the [Reference]({{< relref "docs/reference/_index.md" >}}) for exhaustive configuration details and all available steps
- **Cookbooks**: Browse practical examples and complete workflows in the cookbooks collection
