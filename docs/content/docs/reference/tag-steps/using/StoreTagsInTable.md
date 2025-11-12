---
weight: 50
---

# StoreTagsInTable

Store the tags in a TSV table.

```toml
[[step]]
    action = "StoreTagsInTable"
    infix = "tags"
    compression = "Raw" # Raw, Gzip, Zstd
    region_separator = "_" # (optional) char to separate regions in a tag, if it has multiple
```

This transformation writes all current tags to a tab-separated values (TSV) table file for further analysis.

The output filename is constructed as `{prefix}_{infix}.tsv` (or with custom separator if configured).


### Interaction with demultiplexing
When demultiplexing is used, separate TSV files are created for each barcode: `{prefix}_{infix}_{barcode}.tsv`.
