---
weight: 5
---

# FilterByTag

Remove sequences that have (or don't have) a tag.

```toml
[[step]]
    action = "FilterByTag"
    label = "mytag"
    keep_or_remove = "Keep" # or "Remove"
```

This transformation filters reads based on the presence or absence of a specified tag. Use "Keep" to retain reads that have the tag, or "Remove" to discard reads that have the tag.
