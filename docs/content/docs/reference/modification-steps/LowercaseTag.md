---
weight: 150
---

# LowercaseTag


```toml
[[step]]
    action = "LowercaseTag"
    label = "mytag"

[[step]]
	action = "StoreTagInSequence"
	label = "mytag"
```

Replace the sequence of the tag with it's lowercase version.

Follow with [StoreTagInSequence]({{< relref "docs/reference/tag-steps/using/StoreTagInSequence.md" >}}) to apply the lowercase tag back onto the read.

See [the tag section]({{< relref "docs/reference/tag-steps/_index.md" >}}) for tag generation.
