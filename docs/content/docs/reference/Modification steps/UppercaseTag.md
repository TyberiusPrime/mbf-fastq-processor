---
weight: 180
---

# UppercaseTag


```toml
[[step]]
    action = "UppercaseTag"
    label = "mytag"

# You still want to StoreTagInSequence after this to actually change the sequence.
[[step]]
	action = "StoreTagInSequence"
	label = "mytag"

```

Replace the sequence of the tag with its uppercase version.

See [the tag section](../../tag-steps) for tag generation.
