---
weight: 50
---

# StoreTagInSequence

Store the tag's replacement in the sequence, replacing the original sequence at that location.

```toml
[[step]]
    action = "StoreTagInSequence"
    in_label = "mytag"
    ignore_missing = true # if false, an error is raised if the tag is missing
```

This transformation stores the tag's value back into the sequence, replacing the original sequence at that location.

Note that if this changes the length of the sequence, existing location tags will loose their location data (retaining their sequence though).
