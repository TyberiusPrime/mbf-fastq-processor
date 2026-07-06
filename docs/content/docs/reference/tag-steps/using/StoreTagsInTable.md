---
weight: 50
---

# StoreTagsInTable

Store the tags in a TSV table.

```toml
[[step]]
    action = "StoreTagsInTable"
    infix = "tags"
    compression = "Gzip" # Raw, Gzip, Zstd
    compression_level = 5 # (optional) 0..9 for gzip, zstd 1..22
    region_separator = "_" # (optional) char to separate regions in a tag, if it has multiple
    in_labels = ["mytag", ] # Store just these tags. Optional, all tags store if not set
    include_read_name = true # (optional) include the ReadName column. Default: true
    include_read_comment = false # (optional) include the ReadComment column. Default: false
```

This transformation writes all current tags to a tab-separated values (TSV) table file for further analysis.

The output filename is constructed as `{prefix}_{infix}.tsv` (or with custom separator if configured).

By default all labels are stored, overwrite by setting `in_labels`.

Set `include_read_name = false` to omit the `ReadName` index column from the output.

Set `include_read_comment = true` to include the read's comment (ie. name after
[input.options.read_comment_char]({{< relref "docs/redirects/input-section.md" >}}#input-options)).

### Interaction with demultiplexing
When demultiplexing is used, separate TSV files are created for each barcode: `{prefix}_{infix}_{barcode}.tsv`.

### Interaction with output.chunk_size
No chunking is performed on the generated tables.
