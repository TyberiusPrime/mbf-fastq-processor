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
    segment = "read1"       # Segment to measure length from, or 'All' for total length (default: only segment if single-read input)
```

Divide an existing numeric tag by the read length to produce a normalized rate.

Typical use case: Divide
[EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}})
by the read length to get a fraction (rate).

Output range is 0..=1, assuming the source tag value does not exceed the read length.

Reads with zero length produce a rate of 0.
