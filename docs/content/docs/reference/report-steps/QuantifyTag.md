---
weight: 50
---

# QuantifyTag

Count the occurrences of each tag-sequence.

```toml
[[step]]
    action = "QuantifyTag"
    in_label = "mytag"
    infix = "tagcount" # output file is output{ix_separator}tagcount.qr.json (default '_' → output_tagcount.qr.json)
    region_separator = "_"  # optional. If the tag consists of multiple regions, join them with this string
```

This transformation counts how many times each unique tag value appears and outputs the results to a JSON file.


### Demultiplex interaction

Barcodes are counted per demultiplexed stream.
