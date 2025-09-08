# ExtractGCContent


```toml
[[step]]
    action = "ExtractGCContent"
    target = "Read1" # Read1|Read2|Index1|Index2|All
    label="gc"
```

Count what percentage of bases are GC (as opposed to AT).
Non-AGTC bases (e.g. N) are ignored in both the numerator and denominator.

