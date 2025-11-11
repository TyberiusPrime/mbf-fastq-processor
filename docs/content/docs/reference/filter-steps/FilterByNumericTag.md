---
weight: 10
---

# FilterByNumericTag

Remove molecules by thresholding on numeric tag.

```toml

[[step]]
    action = "CalcLength"
    out_label = "mytag"
    segment = "read1"

[[step]]
    action = "FilterByNumericTag"
    in_label = "mytag"
    keep_or_remove = "Keep" # or "Remove"
    min_value = 5 # >= this, optional
    max_value = 21 # < this, optional
```

The example only keeps reads that are between 5 and 20 bases long.

Consider using an [EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}}) for more complicated decisions.
