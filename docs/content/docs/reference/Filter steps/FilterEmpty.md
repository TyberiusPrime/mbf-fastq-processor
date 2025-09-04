---
weight: 50
---
## FilterEmpty


```toml
[[step]]
    action = "FilterEmpty"
    target = "Read1" # Read1|Read2|Index1|Index2
```

Drop the molecule if the read has length 0.
(Use after other processing.)

A special case of [FilterMinLen](../filterminlen).

This is necessary if your modification can produce 'empty'
reads - downstream aligners like STAR tend to dislike these in their input.

On target='All', only filters reads that are empty in all parts.
Use multiple FilterEmpty to filter if any part is empty.


