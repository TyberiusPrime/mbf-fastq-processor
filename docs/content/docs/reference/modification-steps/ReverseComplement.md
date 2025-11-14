# ReverseComplement

```toml
[[step]]
    action = "ReverseComplement"
    segment = "read1" # Any of your input segments (default: read1)
    if_tag = "mytag"
```

Reverse-complements the read sequence (and reverses the quality).

This supports IUPAC codes (U is complemented to A, so it's not strictly
reversible). Unknown letters are output verbatim.

Useful to combine with [Swap]({{< relref "docs/reference/modification-steps/Swap.md" >}}).

Optionally only swaps if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy.
