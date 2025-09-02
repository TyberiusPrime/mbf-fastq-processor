---
weight: 50
---

# ExtractRegex

Extract a regexp result. Stores an empty string if not found.

```toml
[[step]]
    action = "ExtractRegex"
    label = "mytag"
    search = "^CT(..)CT"
    replacement = "$1"  # standard regex replacement syntax
    target = "Read1" # where we are searching it
```

This transformation searches for a regular expression pattern in the specified read and extracts the matching portion as a tag.