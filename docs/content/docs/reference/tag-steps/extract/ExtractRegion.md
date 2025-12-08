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
    source = "read1" # Any of your input segments, name:<segment_name> or tag:<any_location_or_string_tag>
    anchor = "Start" # 'Start' or 'End' - start is relative to this
    out_label = "umi"
```

This transformation extracts a fixed-length region from the specified read at a
given position and stores it as a tag.

Start positions are 0-based.

End positions require negative starts (as in python, start=-1, length=1 is the last character).

Use [ExtractRegions]({{< relref
"docs/reference/tag-steps/extract/ExtractRegions.md" >}}) if your region is
actually multiple regions (possibly from different segments).

If the read is shorter than requested, the region will be shorter (and might be
of varying length).
