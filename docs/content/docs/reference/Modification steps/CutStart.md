### CutStart


```toml
[[step]]
    action = "CutStart"
    n = # positive integer, cut n nucleotides from the start of the read
    target = Read1|Read2|Index1|Index2 
```

Cut nucleotides from the start of the read.

May produce empty reads, filter those with [FilterEmptyReads](../../filter-steps/filterempty).

