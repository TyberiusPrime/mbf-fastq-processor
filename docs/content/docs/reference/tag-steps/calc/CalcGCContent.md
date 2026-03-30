---
title: Calc GC Content
---
# CalcGCContent

```toml
[[step]]
    action = "CalcGCContent"
    segment = "read1" # Any of your input segments, or 'All'
    out_label = "gc"
    relative = true   # a rate (true) or a count (false)
```

Count what percentage of bases are GC (as opposed to AT).
Non-AGTC bases (e.g. N) are ignored.

Output is 0..100.


Wrapper around [CalcBaseContent]({{< relref "docs/reference/tag-steps/calc/CalcBaseContent.md" >}}) with `bases = "GC", ignore="N", relative=true`).
