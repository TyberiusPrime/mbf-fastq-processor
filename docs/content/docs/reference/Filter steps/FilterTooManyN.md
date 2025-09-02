# FilterTooManyN


```toml
[[step]]
    action = "FilterTooManyN"
    n = 5 # positive integer, the maximum number of Ns allowed
    target = "Read1" # Read1|Read2|Index1|Index2
```

Filter by the count of N in a read.


## Corresponding options in other software #
- fastp: --n_base_limit

