# FilterSample


```toml
[[step]]
    action = "FilterSample"
    p = 0.5 # float, the chance for any given read to be kept
              # 0..1
    seed = 42 # u64, the seed for the random number generator
    target = "Read1" # Read1|Read2|Index1|Index2
```

Randomly sample a percentage of reads.
Requires a random seed, so always reproducible
