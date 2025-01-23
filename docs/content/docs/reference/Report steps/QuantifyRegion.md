---
weigth: 20
---
# QuantifyRegion

Quantify kmers in regions of the read.
Useful to hunt for (cell) barcodes.

The regions are concatenated with a separator.

```toml

[[transform]]
    action = 'QuantifyRegions'
    infix = 'kmer' # output_filename is output.prefix_infix.qr.json
    regions = [
        {source = "Read1", start = 0, length = 6},
        {source = "Read1", start = 12, length = 6},
    ]
    separator = "-" # defaults to "_"
```

