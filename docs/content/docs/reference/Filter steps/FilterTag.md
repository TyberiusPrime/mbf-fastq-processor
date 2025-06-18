# FilterTag

```toml
[[steps]]
    action = "FilterTag"
    label = "mytag"
    keep_or_remove: Keep | Remove


```

Filters reads that had the tag `mytag` set.

Wether these reads are kept, or removed depends on the `keep_or_remove` option.


See [the tag section](../../tag-steps) for tag generation.


