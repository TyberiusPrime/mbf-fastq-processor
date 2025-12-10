---
weight: 160
---

# LowercaseSequence


```toml
[[step]]
    action = "LowercaseSequence"
    segment = "read1" # Any of your input segments, or 'All'
    if_tag = "mytag"

```

Convert the complete sequence to lowercase.

Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy.
