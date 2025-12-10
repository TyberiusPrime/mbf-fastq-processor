---
weight: 170
---

# UppercaseSequence


```toml
[[step]]
    action = "UppercaseSequence"
    segment = "read1" # Any of your input segments, or 'All'
    if_tag = "mytag"

```

Convert the complete sequence to uppercase.

Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy.

