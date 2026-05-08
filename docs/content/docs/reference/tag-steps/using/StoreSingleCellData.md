---
weight: 55
---

# StoreSingleCellData

Collect per-read (gene, cell barcode, UMI) index triples, sort by (gene, cell),
in memory and write a compact row-oriented binary file and two lookup tables to disk.

Useful for later transformation into a single cell expression matrix
after UMI deduplication.

```toml
[[step]]
    action = "StoreSingleCellData"
    cell_tag = "cell_bc"           # tag carrying the cell-barcode sequence or label
    gene_tag = "gene_bc"           # tag carrying the gene-barcode sequence
    umi_tag = "umi"                # tag carrying the raw UMI sequence
    cell_barcodes = "cells"        # [barcodes.cells] section (sequence → name)
    gene_barcodes = "genes"        # [barcodes.genes] section (sequence → name)
    tag_contains_barcode = true    # (optional) see below; auto-detected if omitted
    infix = ""                     # (optional) filename infix
    compression = "Raw"            # (optional) Raw, Gzip, Zstd — for all output files.
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

### tag_contains_barcode

Controls how `cell_tag` values are resolved:

| Value | Behaviour |
|-------|-----------|
| `true` | Value is a **barcode sequence**; looked up in `[barcodes.cells]` by sequence |
| `false` | Value is a **corrected label** (e.g. from `AssignByHalves`); looked up by name |
| *(omitted)* | Auto-detected: `Location` tags → `true`, `String` tags → `false` |

Gene barcodes are always looked up by sequence.

Unrecognised sequences are assigned index 0 ("unmatched"). 
Real barcodes are 1-indexed in the order they appear in the `[barcodes.*]` table.

`umi_tag` accepts `Location` or `String`. The UMI is 2-bit encoded (A=0, C=1,
G=2, T=3; any other → `u32::MAX`). Maximum UMI length is 16 bp.

## Output files

Three files are written per run:

| File | Description |
|------|-------------|
| `{prefix}_{infix}scd.bin` | Sorted row-binary data (see format below) |
| `{prefix}_{infix}scd.cell_barcodes.txt` | Cell name lookup (line 0 = "unmatched") |
| `{prefix}_{infix}scd.genes.txt` | Gene name lookup (line 0 = "unmatched") |

The `.bin` file format is:

```
[magic: 8 bytes "FQRSCD\x00\x02"]
[num_rows: u32 LE]
[umi_length: u8]
[gene_idx: u32 LE | cell_idx: u32 LE | umi: u32 LE]  × num_rows
```

`umi_length` is the maximum UMI length seen (in bp, maximum length: 16 Longer will lead to a runtime error). 
Rows are sorted by `(gene_idx, cell_idx)`. 

Read in Python with:

```python
import numpy as np, struct
data = open("output_scd.bin", "rb").read()
umi_length = data[12]
rows = np.frombuffer(data[13:], dtype=np.uint32).reshape(-1, 3)
# rows[:, 0] = gene_idx, rows[:, 1] = cell_idx, rows[:, 2] = umi (2-bit encoded)
```

The lookup `.txt` files have one name per line; the line number is the index
used in the binary file (line 0 is always "unmatched").


There is a helper script in the fastqrab repository/dev/view_scd.py to inspect the data.

## Interaction with demultiplexing

When a `Demultiplex` step precedes this step, a separate `.bin` file is written
for each barcode group. The lookup tables are shared (singleton) across all
groups:

```
{prefix}_{infix}scd_{sample_name}.bin   # one per demultiplex group
{prefix}_{infix}scd.cell_barcodes.txt
{prefix}_{infix}scd.genes.txt
```
