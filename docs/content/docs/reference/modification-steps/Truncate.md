# MaxLen

```toml
[[step]]
    action = "Truncate"
    n = 100 # the maximum length of the read. Cut at end if longer
    segment = "read1" # Any of your input segments (default: read1)
    if_tag = "mytag"
```

Cut the read down to `n' bases.

Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy.
