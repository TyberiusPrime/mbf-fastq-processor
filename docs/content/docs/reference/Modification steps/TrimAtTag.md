---
weight: 50
---

# TrimAtTag

Trim the read at the position of a tag.

```toml
[[step]]
    action = "TrimAtTag"
    label = "mytag"
    direction = "Start" # or "End"
    keep_tag = false # if true, the tag sequence is kept in the read
```

This transformation trims the read at the position where a tag was found.
The `direction` parameter determines whether to trim from the start or end of the tag,
and `keep_tag` determines whether the tag sequence itself is retained.
