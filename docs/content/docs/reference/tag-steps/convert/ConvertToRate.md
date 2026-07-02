---
title: Convert To Rate
---
# ConvertToRate

```toml
[[step]]
    action = "CalcBaseContent"
    segment = "read1"
    bases_to_count = "A"
    relative = false
    out_label = "a_count"  # absolute A-base count

[[step]]
    action = "ConvertToRate"
    in_label = "a_count"    # The numeric tag to divide by read length
    out_label = "a_rate"    # Output tag label
    segment = "read1" # Any of your input segments, or 'All' 
                     # measure length from this segment, 
                     # or total length if 'All' (default: only segment if single-read input, otherwise required)
```

Divide an existing numeric tag by the read length to produce a normalized rate.

Typical use case: Divide
[EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}})
by the read length to get a fraction (rate).

Output range is 0..=1, assuming the source tag value does not exceed the read length.

Reads with zero length produce a rate of 0.
