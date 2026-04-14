---
weight: 5
title: "Barcodes section"
---

# `[barcodes.*]` section

Barcode tables supply the sequence-to-sample-name mappings used by
[Demultiplex]({{< relref "docs/reference/Demultiplex.md" >}}) and
[HammingCorrect]({{< relref "docs/reference/tag-steps/using/HammingCorrect.md" >}}).

Each table is an independent named dictionary.  The name is chosen by the
user and referenced from the step that consumes it.

```toml
# ignore_in_test
[barcodes.my_barcodes]
AAAAAA = "sample-1"
CCCCCC = "sample-2"
GGGGGG = "sample-3"
```

The table may appear anywhere in the TOML file; forward and backward
references are both valid.

## Key format

Keys are DNA sequences using uppercase IUPAC nucleotide codes.
All standard IUPAC ambiguity codes are accepted (e.g. `N`, `R`, `Y`, `W`, …).

A `_` in a key separates regions when the tag being matched spans multiple
extracted segments joined with `_`:

```toml
# ignore_in_test
[barcodes.dual_index]
AAAAAA_TTTTTT = "sample-1"   # i7 = AAAAAA, i5 = TTTTTT
CCCCCC_GGGGGG = "sample-2"
```

## Values

Values are arbitrary sample-name strings that appear in the output file names
produced by [Demultiplex]({{< relref "docs/reference/Demultiplex.md" >}}).
The name `no-barcode` is reserved and will be rejected.

Multiple keys may map to the same value (barcode aliases):

```toml
# ignore_in_test
[barcodes.my_barcodes]
AAAAAA = "sample-1"
AAAAAC = "sample-1"   # treated identically to AAAAAA
CCCCCC = "sample-2"
```

## Constraints

| Constraint | Detail |
|---|---|
| Non-empty | At least one entry required per table. |
| Uniform length | All keys in the same table must have the same length (counting `_` separators). |
| Non-overlapping IUPAC | Two keys must not match any of the same concrete sequences. e.g. `NNNN` and `ATCG` overlap. |
| Reserved name | The value `"no-barcode"` is used internally for unmatched reads and may not be used as a sample name. |

## Multiple tables

Define as many `[barcodes.<name>]` sections as needed; each step refers to
the one it needs by name:

```toml
# ignore_in_test
[barcodes.i7_barcodes]
AAAAAA = "sample-1"
CCCCCC = "sample-2"

[barcodes.i5_barcodes]
TTTTTT = "lib-A"
GGGGGG = "lib-B"
```

## See also

- [Demultiplex]({{< relref "docs/reference/Demultiplex.md" >}}) — how barcode tables are consumed to fork the output
- [HammingCorrect]({{< relref "docs/reference/tag-steps/using/HammingCorrect.md" >}}) — correct sequencing errors in a tag before demultiplexing
- [ExtractRegion]({{< relref "docs/reference/tag-steps/extract/ExtractRegion.md" >}}) — extract a barcode from a read into a tag
