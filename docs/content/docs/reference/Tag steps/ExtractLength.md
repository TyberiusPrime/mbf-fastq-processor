---
weight: 50
---

# ExtractLength

Extract the length of a read as a tag.

```toml
[[step]]
    action = "ExtractLength"
    label = "mytag"
    target = "Read1" # Read1|Read2|Index1|Index2|All
```

This transformation creates a tag containing the length of the specified read.