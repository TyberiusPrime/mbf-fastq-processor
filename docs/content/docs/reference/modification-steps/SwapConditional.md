# SwapConditional

```toml
[[step]]
    action = "SwapConditional"
    in_label = "mytag"  # Boolean tag that determines whether to swap
    segment_a = "read1"       # Optional
    segment_b = "read2"       # Optional
```

Conditionally swaps exactly two segments based on a boolean tag value.

For each read, if the boolean tag is `true`, the segments are swapped for that specific read.
If the tag is `false`, the read is left unchanged.

This is useful for conditionally reorganizing paired-end data based on read properties,
such as swapping reads based on length, quality, or other criteria determined by prior steps.

## Parameters

- `in_label`: (required) The name of a boolean tag that determines whether to swap each read
- `segment_a`/`segment_b`: (optional) Only necessary if there are more than two segments defined in the input

## Example

```toml
# Create boolean tag: swap if read1 is longer than read2
[[step]]
    action = "EvalExpression"
    out_label = "swap_if_read1_longer"
    expression = "len_read1 > len_read2" # len_ tags are virtual
    result_type = "bool"

# Conditionally swap reads
[[step]]
    action = "SwapConditional"
    in_label = "swap_if_read1_longer"
```


Note that this does not reverse complement. You might want to use [ReverseComplementConditional]({{< relref "docs/reference/modification-steps/ReverseComplementConditional.md" >}}) in conjunction.
