---
weight: 55
---

# StoreSingleCellMatrix

Collect per-read (gene, cell barcode, UMI) index triples in memory,
and an output [mtx/MatrixMarket](https://math.nist.gov/MatrixMarket/formats.html) formatted count matrix.


```toml
[[step]]
    action = "StoreSingleCellMatrix"
    cell_tag = "cell_bc"           # tag carrying the cell-barcode sequence or label
    gene_tag = "gene_bc"           # tag carrying the gene-barcode sequence
    umi_tag = "umi"                # tag carrying the raw UMI sequence
    cell_barcodes = "cells"        # [barcodes.cells] section (sequence → name)
    gene_barcodes = "genes"        # [barcodes.genes] section (sequence → name)
    cell_tag_contains_barcode = true    # (optional) see below; auto-detected if omitted
    gene_tag_contains_barcode = true    # (optional) see below; auto-detected if omitted
    infix = ""                     # (optional) filename infix. 
    compression = "Raw"            # (optional) Raw, Gzip, Zstd — for all output files.
    umi_aggregation = "Exact"       # How to handle duplicate UMIs. See below
```

## Inputs

`cell_tag` must carry either a `Location` tag (from `ExtractRegion`) or a `String` tag
whose value is either a barcode sequence or a corrected label (e.g. from 
`[AssignByHalves]({{< relref "docs/redirects/AssignByHalves.md" >}})`
or 
`[HammingCorrect]({{< relref "docs/redirects/HammingCorrect.md" >}})` with output = 'label'
).

`gene_tag` must carry a `Location` or `String` tag whose value is a barcode sequence.

Both are looked up against the corresponding `[barcodes.*]` section.

### (cell|gene)_tag_contains_barcode

Controls how `cell_tag` or `gene_tag` values are resolved:

| Value | Behaviour |
|-------|-----------|
| `true` | Value is a **barcode sequence**; looked up in by sequence |
| `false` | Value is a **corrected label** (e.g. from `AssignByHalves`); looked up by name |
| *(omitted)* | Auto-detected: `Location` input tags → `true`, `String` tags → `false` |

Unrecognised sequences are assigned index 0 ("unmatched"). 

Real barcodes are 1-indexed in the order they appear in the `[barcodes.*]` table.

`umi_tag` accepts `Location` or `String`. The UMI is 2-bit encoded (A=0, C=1,
G=2, T=3; any other → `u32::MAX`). Maximum UMI length is 16 bp.


## UMI aggregation

Depending on the `umi_aggregation` setting, different UMI->count algorithms are used.


Possible Values:

* None - Do not aggregate UMIs, report read count
* Exact - Each UMI counts as most once.


## Output files

Three files are written per run:

| File | Description |
|------|-------------|
| `{prefix}_{infix}scd.matrix.mtx(.gz)` | matrix market file|
| `{prefix}_{infix}scd.barcodes.txt(.gz)` | Cell name / barcode lookup (line 0 = "unmatched") |
| `{prefix}_{infix}scd.features.txt(.gz)` | Gene name lookup (line 0 = "unmatched") |


## Interaction with demultiplexing

When a `Demultiplex` step precedes this step, a separate `.mtx` file is written
for each barcode group. The lookup tables are shared (singleton) across all
groups:

```
{prefix}_{infix}_scd_{sample_name}.mtx   # one per demultiplex group
{prefix}_{infix}_scd.barcodes.txt
{prefix}_{infix}_scd.features.txt
```
