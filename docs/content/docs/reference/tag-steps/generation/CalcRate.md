---
weight: 56
---

# CalcRate

Derive a numeric rate tag (= value / base count) from an existing numeric tag, optionally using a custom denominator tag or logarithmic scaling.

```toml
[[step]]
    action = "ExtractBaseContent"
    bases_to_count = "AGTC"
    segment = "read1"
    label = "non_n"

[[step]]
    action = "ExtractBaseContent"
    bases_to_count = "N"
    segment = "read1"
    label = "n_count"

[[step]]
    action = "CalcRate"
    label = "n_rate"
    nominator_label = "n_count"       # upstream numeric tag
    denominator_label = "non_n"  # optional, defaults to read length
    # segment = 'read1'          # or 'All'; optional if denominator_label is set and only one segment is defined.
    # log_base = "e"                   # optional: e, 2, or 10
    # log_offset = 1.0                 # optional: added before logging
```

- `nominator_label` must reference an upstream numeric tag.
- When `denominator_label` is omitted, the read length of the specified segment is used.
- Setting `log_base` switches the calculation to `log(nominator + offset) - log(denominator + offset)`; use `log_offset = 1` with `log_base = "e"` to benefit from [`ln_1p`](https://doc.rust-lang.org/std/primitive.f64.html#method.ln_1p) precision.
