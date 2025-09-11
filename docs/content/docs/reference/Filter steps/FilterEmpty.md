---
weight: 50
---
## FilterEmpty


```toml
[[step]]
    action = "FilterEmpty"
    segment = "read1" # Any of your input segments, or 'All'
```

Drop the molecule if the read has length 0.
(Use after other processing.)

A special case of [FilterMinLen](../filterminlen).

This is necessary if your modification can produce 'empty'
reads - downstream aligners like STAR tend to dislike these in their input.

On segment='All', only filters reads that are empty in all parts.
Use multiple FilterEmpty to filter if any part is empty.