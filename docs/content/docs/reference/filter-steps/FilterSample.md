# FilterSample


```toml
[[step]]
    action = "FilterSample"
    p = 0.5 # float, the chance for any given read to be kept
              # 0..1
    seed = 42 # (optional) random seed for reproducibility
```

Randomly sample a percentage of reads.
Requires a random seed to ensure reproducibility.
