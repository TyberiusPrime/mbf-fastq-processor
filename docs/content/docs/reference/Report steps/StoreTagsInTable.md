---
weight: 50
---

# StoreTagsInTable

Store the tags in a TSV table.

```toml
[[step]]
    action = "StoreTagsInTable"
    table_filename = "tags.tsv"
    compression = "Raw" # Raw, Gzip, Zstd
    region_separator = "_" # (optional) char to separate regions in a tag, if it has multiple
```

This transformation writes all current tags to a tab-separated values (TSV) table file for further analysis.