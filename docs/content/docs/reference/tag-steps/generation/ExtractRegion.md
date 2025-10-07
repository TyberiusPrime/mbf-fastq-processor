---
weight: 50
---

# ExtractRegion

Extract a fixed position region.

```toml
[[step]]
    action = "ExtractRegion"
    start = 5
    length = 8
    segment = "read1" # Any of your input segments
    label = "umi"
```

This transformation extracts a fixed-length region from the specified read at a given position and stores it as a tag.

Use [ExtractRegions]({{< relref "docs/reference/tag-steps/generation/ExtractRegions.md" >}}) if your region is actually multiple regions (possibly from different segments).
