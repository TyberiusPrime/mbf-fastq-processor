---
weight: 150
---

# AddTagSequencToName


```toml
[[step]]
    action = "AddTagSequencToName"
    label = "mytag"

```

Add a 'comment' to the read name, by appending " mytag=<sequence>" to it.
Note the space in front.

If mytag's extract did not hit, add 'mytag='.


See [the tag section](../../tag-steps) for tag generation.

