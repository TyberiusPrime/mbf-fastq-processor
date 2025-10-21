---
weight: 50
---

# ForgetAllTags

Remove every tag currently stored for the read batch.

```toml
[[step]]
    action = "ForgetAllTags"
```

Use this when you want to clear all tag labels before continuing.
It is handy after persisting tags to an external table, or before running
steps that must not see previously extracted metadata.
