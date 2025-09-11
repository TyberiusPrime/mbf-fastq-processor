---
weight: 50
---

# ExtractLength

Extract the length of a read as a tag.

```toml
[[step]]
    action = "ExtractLength"
    label = "mytag"
    segment = "read1" # Any of your input segments, or 'All'
```

This transformation creates a tag containing the length of the specified read.