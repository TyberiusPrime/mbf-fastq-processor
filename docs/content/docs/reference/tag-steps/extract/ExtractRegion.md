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
    out_label = "umi"
```

This transformation extracts a fixed-length region from the specified read at a given position and stores it as a tag.

Start positions are 0-based.

Use [ExtractRegions]({{< relref "docs/reference/tag-steps/extract/ExtractRegions.md" >}}) if your region is actually multiple regions (possibly from different segments).

If the read is shorter than requested, the region will be shorter (and might be of varying length).
