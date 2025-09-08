# ExtractNCount


```toml
[[step]]
    action = "ExtractNCount"
    target = "Read1" # Read1|Read2|Index1|Index2|All
    label="ncount"
```

Count how many N are present in the read


## Corresponding options in other software #
- fastp: --n_base_limit (if combined with FilterByNumericTag)
