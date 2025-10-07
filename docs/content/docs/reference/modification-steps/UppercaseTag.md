---
weight: 180
---

# UppercaseTag


```toml
[[step]]
    action = "UppercaseTag"
    label = "mytag"

[[step]]
	action = "StoreTagInSequence"
	label = "mytag"

```

Replace the sequence of the tag with its uppercase version.

Follow with [StoreTagInSequence]({{< relref "docs/reference/tag-steps/using/StoreTagInSequence.md" >}}) to apply the uppercase tag back onto the read.

See [the tag section]({{< relref "docs/reference/tag-steps/_index.md" >}}) for tag generation.
