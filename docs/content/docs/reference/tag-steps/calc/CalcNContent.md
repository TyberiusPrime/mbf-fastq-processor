---
title: CalcNContent
---

# CalcNCount

```toml
[[step]]
    action = "CalcNContent"
    segment = "read1" # Any of your input segments, or 'All'
    out_label = "ncount"
    relative = true   # a rate (true) or a count (false)
```

Count how many N are present in the read.

This step is a convenient wrapper for
[`CalcBaseContent`](./CalcBaseContent.md) with `bases_to_count = "N"`. 

## Corresponding options in other software

- fastp: --n_base_limit (if combined with [FilterByNumericTag]({{< relref "docs/reference/filter-steps/FilterByNumericTag.md" >}}))
