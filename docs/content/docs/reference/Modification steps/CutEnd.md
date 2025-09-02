### CutEnd

```toml
[[step]]
    action = "CutEnd"
    n = 5 # positive integer, cut n nucleotides from the end of the read
    target = "Read1" # Read1|Read2|Index1|Index2 (default: read1)
```

Cut nucleotides from the end of the read.

May produce empty reads, filter those with [FilterEmptyReads](../../filter-steps/filterempty).
