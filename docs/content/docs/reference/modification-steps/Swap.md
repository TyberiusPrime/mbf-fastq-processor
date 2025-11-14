# Swap

```toml
[[step]]
    action = "Swap"
    segment_a = "read1"
    segment_b = "read2"
    if_tag = "mytag"
```


Swaps exactly two segments.

Arguments `segment_a`/`segment_b` are only necessary if there are more than two segments defined in the input.

Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy.

