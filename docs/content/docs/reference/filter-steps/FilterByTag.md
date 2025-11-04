---
weight: 5
---

# FilterByTag

This transformation filters molecules based on the presence or absence of a specified tag. 

Use "Keep" to retain molecules that have the tag, or "Remove" to discard reads that have the tag.

If used on a boolean tag, the boolean value of the tag is used to determine whether to keep or remove the read.

For numeric tags, use [FilterByNumericTag]({{< relref "docs/reference/filter-steps/FilterByNumericTag.md" >}}).


```toml
[[step]]
    action = "FilterByTag"
    label = "mytag"
    keep_or_remove = "Keep" # or "Remove"
```

