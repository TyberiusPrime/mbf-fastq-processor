---
weight: 55
---

# CalcExpectedError

Compute the sum of per-base error probabilities (expected errors) for each read assuming PHRED+33 qualities.

```toml
[[step]]
    action = "CalcExpectedError"
    label = "expected_error"
    segment = "read1" # Any of your input segments, or 'All'
```

If your data is not encoded as PHRED+33, convert it first (for example, with `ConvertQuality`) before running this step.
