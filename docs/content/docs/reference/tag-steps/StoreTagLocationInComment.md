---
weight: 50
---

# StoreTagLocationInComment

Store the coordinates of a tag in the comment (start-end, 0-based, half-open).

```toml
[[step]]
    action = "StoreTagLocationInComment"
    label = "mytag"
    segment = "read1" # Any of your input segments, or 'All'
    comment_insert_char = " " # (optional) char at which to insert comments
    comment_separator = "|" # (optional) char to separate comments
```

This transformation stores the location coordinates of a tag as a comment in the read name, useful for tracking where tags were extracted from.