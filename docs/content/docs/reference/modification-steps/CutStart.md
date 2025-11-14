### CutStart


```toml
[[step]]
    action = "CutStart"
    n = 5 # positive integer, cut n nucleotides from the start of the read
    segment = "read1" # Any of your input segments
    if_tag = "mytag"
```

Cut nucleotides from the start of the read.

May produce empty reads; filter those with [FilterEmpty]({{< relref "docs/reference/filter-steps/FilterEmpty.md" >}}).

Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy.
