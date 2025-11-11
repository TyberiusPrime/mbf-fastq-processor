---
weight: 50
---

# CalcLength

Extract the length of a read as a tag.

```toml
[[step]]
    action = "CalcLength"
    out_label = "mytag"
    segment = "read1" # Any of your input segments, or 'All'
```

This transformation creates a tag containing the length of the specified read.
