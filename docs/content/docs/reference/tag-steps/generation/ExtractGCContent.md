---
title: Extract GC Content
---
# ExtractGCContent

```toml
[[step]]
    action = "ExtractGCContent"
    segment = "read1" # Any of your input segments, or 'All'
    label = "gc"
```

Count what percentage of bases are GC (as opposed to AT).
Non-AGTC bases (e.g. N) are ignored in both the numerator and denominator.

Output is 0..100.


Wrapper around [ExtractBaseContent]({{< relref "docs/reference/extract-steps/ExtractBaseContent.md" >}}) with `bases = "GC", ignore="N", relative=true`).
