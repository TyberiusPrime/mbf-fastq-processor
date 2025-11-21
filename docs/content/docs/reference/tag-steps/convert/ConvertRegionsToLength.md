---
weight: 58
---

# ConvertRegionsToLength

Turn region tags (such as those produced by `ExtractRegion`/`ExtractRegions`) into numeric length tags.

```toml
[[step]]
    action = "ExtractRegion"
    out_label = "adapter"
    source = "read1"
    start = 0
    len = 12
    anchor = "Start"

[[step]]
    action = "ConvertRegionsToLength"
    out_label = "adapter_len"
    in_label = "adapter"
```

- The new tag stores the total span (in bases) covered by all regions on each read.
- Reads without the source tag receive a length of `0`.
- `label` must be different from `region_label`; the step keeps the original region tag.
