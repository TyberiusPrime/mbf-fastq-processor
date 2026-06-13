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

Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy via `if_tag`.

## Conditional swaps forget location tags

A location tag records *where* in a segment some bytes live, so it is tied to one
segment. An **unconditional** swap moves both segments as whole units, so location
tags survive — they simply follow their segment.

A **conditional** swap (`if_tag` set) swaps reads one at a time, so after it runs a
single location tag would point into *two different* segments depending on the read.
Loctations are limted to a single segment, so a conditional `Swap` **forgets every location tag on
the two swapped segments**. Referring to such a tag (or its `location_…` /
`initial_location_…` companion) after the swap is a configuration error.

If you need to keep the data across a conditional swap, snapshot the tag to a plain
string **before** the swap with
[`ConcatTags`]({{< relref "docs/reference/tag-steps/using/ConcatTags.md" >}}); string
tags are not segment-bound and are unaffected by the swap.

