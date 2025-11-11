---
weight: 50
---

# ExtractRegions

Extract from multiple fixed position regions.

```toml
[[step]]
    action = "ExtractRegions"
    regions = [
        {segment = "read1", start = 0, length = 8},
        {segment = "read1", start = 12, length = 4},
    ]
    out_label = "barcode"
```

This transformation extracts multiple fixed-length regions from reads and concatenates them into a single tag.


ExtractRegions with only one region are exactly equivalent to [ExtractRegion]({{< relref "docs/reference/tag-steps/extract/ExtractRegion.md" >}}).
