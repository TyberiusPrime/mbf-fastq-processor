---
weight: 150
---

# LowercaseTag


```toml
[[step]]
    action = "LowercaseTag"
    label = "mytag"

# You still want to StoreTagInSequence after this to actually change the sequence.
[[step]]
	action = "StoreTagInSequence"
	label = "mytag"
```

Replace the sequence of the tag with it's lowercase version.

See [the tag section](../../tag-steps) for tag generation.


