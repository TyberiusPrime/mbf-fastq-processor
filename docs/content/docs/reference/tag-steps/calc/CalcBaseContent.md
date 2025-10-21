---
title: Calc Base Content
---
# CalcBaseContent

```toml
[[step]]
    action = "CalcBaseContent"
    segment = "read1" # Any of your input segments, or 'All'
    label = "at_content"
    bases_to_count = "AT"
    bases_to_ignore = "N"
    relative = true # default
```

Counts the percentage of bases that match `bases_to_count`, while removing any bases listed in `bases_to_ignore` from the denominator (if relative = True). 

Both lists are case-insensitive, and accept only ascii letters. When no bases remain after filtering, the step returns `0`.

Set `relative = false` to emit absolute base counts instead of percentages. Absolute mode requires `bases_to_ignore` to remain unset, otherwise the configuration check fails.

Use this in combination with `StoreTagInComment` or `FilterByNumericTag` to surface the computed percentages downstream.
