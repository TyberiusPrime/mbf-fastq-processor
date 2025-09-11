### CutStart


```toml
[[step]]
    action = "CutStart"
    n = 5 # positive integer, cut n nucleotides from the start of the read
    segment = "read1" # Any of your input segments
```

Cut nucleotides from the start of the read.

May produce empty reads, filter those with [FilterEmptyReads](../../filter-steps/filterempty).