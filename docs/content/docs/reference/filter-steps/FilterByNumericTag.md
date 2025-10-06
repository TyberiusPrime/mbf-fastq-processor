---
weight: 51
---

# FilterByNumericTag

Remove sequences that exceed thresholds on a numeric tag.

```toml

[[step]]
    action = "ExtractLength"
    label = "mytag"
    segment = "read1"

[[step]]
    action = "FilterByNumericTag"
    label = "mytag"
    keep_or_remove = "Keep" # or "Remove"
    min_value = 5 # >= this, optional
    max_value = 21 # < this, optional
```

The example only keeps reads that are between 5 and 20 bases long.
