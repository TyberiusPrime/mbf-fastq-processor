---
weight: 55
---

# CalcExpectedError

Compute aggregated per-base error probabilities (expected errors) for each read assuming PHRED+33 qualities.

```toml
[[step]]
    action = "CalcExpectedError"
    label = "expected_error"
    aggregate = "sum" # or "max"
    segment = "read1" # Any of your input segments, or 'All'
```

If your data is not encoded as Phred+33, convert it first (for example, with `ConvertQuality`) before running this step. Values outside of the Phred+33 range will lead to an stop with an error.

Set `aggregate = "sum"` to calculate the sum of per-base error probabilities. 
Use `aggregate = "max"` to store only the worst base's error probability for each read or read pair.

The later is inspired by (Edgar and Flyvbjerg, 2015)[https://doi.org/10.1093/bioinformatics/btv401.].



