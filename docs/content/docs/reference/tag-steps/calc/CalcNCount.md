---
title: Calc N Count
---

# CalcNCount

```toml
[[step]]
    action = "CalcNCount"
    segment = "read1" # Any of your input segments, or 'All'
    out_label="ncount"
```

Count how many N are present in the read.

This step is a convenient wrapper for
[`CalcBaseContent`](./CalcBaseContent.md) with `bases_to_count = "N"` and
`relative = false`.

## Corresponding options in other software

- fastp: --n_base_limit (if combined with [FilterByNumericTag]({{< relref "docs/reference/filter-steps/FilterByNumericTag.md" >}}))
