---
weight: 50
---

# FilterMinLen


```toml
[[step]]
    action = "FilterMinLen"
    n = 0 # positive , minimum length
    target = "Read1" # Read1|Read2|Index1|Index2
```

Drop the molecule if the read is below a specified length.


## Corresponding options in other software

-  Trimmomatic: MINLEN
- fastp: --length_required

