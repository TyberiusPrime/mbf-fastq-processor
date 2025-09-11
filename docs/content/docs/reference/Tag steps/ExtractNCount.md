# ExtractNCount


```toml
[[step]]
    action = "ExtractNCount"
    segment = "read1" # Any of your input segments, or 'All'
    label="ncount"
```

Count how many N are present in the read


## Corresponding options in other software #
- fastp: --n_base_limit (if combined with FilterByNumericTag)