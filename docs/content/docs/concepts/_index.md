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

## Steps and targets

Every step sees whole fragments so paired segments stay in lock-step: if you filter a fragment based on `read1`, the associated `read2` and any index reads disappear alongside it.

Many steps accept a `segment` argument to restrict their work to a specific input stream, while still retaining awareness of the whole fragment.

Tag-generating steps must be paired with consumers. mbf-fastq-processor will error if a label is produced but never used, helping you catch typos early.


## Demultiplexing
Demultiplexing splits the fragment stream into multiple outputs, e.g. on 'barcodes', on length, or on any tag.

## Further reading

Continue with the [Reference]({{< relref "docs/reference/_index.md" >}}) for exhaustive configuration details, or explore integration scenarios in the How-To collection.
