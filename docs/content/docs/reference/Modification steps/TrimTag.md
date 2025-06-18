# TrimTag


```toml
[[steps]]
    action = "TrimTag"
    label = "mytag"
    direction: Start|End"
    keep_tag: bool


```

Remove either sequence before (direction="Start") or after (direction="End") the tag sequence.
Depending on `keep_tag` the tag sequence itself is also removed.

This always modifies the target that was used to define the tag.


See [the tag section](../../tag-steps) for tag generation.


