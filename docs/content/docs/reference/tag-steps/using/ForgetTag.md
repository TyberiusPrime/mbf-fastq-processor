---
weight: 50
---

# ForgetTag

Forget about a tag. Useful if you want to store tags in a table, but not this one.

```toml
[[step]]
    action = "ForgetTag"
    in_label = "mytag"
```

This transformation removes a specified tag from the molecule's tag collection.
