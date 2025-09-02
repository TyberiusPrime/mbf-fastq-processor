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
    source = "Read1" # Read1|Read2|Index1|Index2
    label = "umi"
```

This transformation extracts a fixed-length region from the specified read at a given position and stores it as a tag.