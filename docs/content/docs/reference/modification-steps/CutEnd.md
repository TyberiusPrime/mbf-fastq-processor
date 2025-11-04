# CutEnd

```toml
[[step]]
    action = "CutEnd"
    n = 5 # positive integer, cut n nucleotides from the end of the read
    segment = "read1" # Any of your input segments (default: read1)
```

Cut nucleotides from the end of the read.

May produce empty reads; filter those with [FilterEmpty]({{< relref "docs/reference/filter-steps/FilterEmpty.md" >}}).
