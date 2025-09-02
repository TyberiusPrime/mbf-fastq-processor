# FilterLowComplexity


```toml
[[step]]
    action = "FilterLowComplexity"
    threshold = 0.3 # Complexity must be >= this threshold (0..1).
                    # 0.30 might be a good value, which means 30% complexity is required.
    target = "Read1" # Read1|Read2|Index1|Index2
```


Filter low complexity reads. 

Based on the percentage of bases that are changed form their predecessor.

## Corresponding options in other software 
- fastp: -low_complexity_filter
