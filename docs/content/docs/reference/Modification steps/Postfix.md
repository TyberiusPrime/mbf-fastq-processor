---
weight: 50
---

# Postfix

Add DNA to the end of read sequences.

```toml
[[step]]
    action = "Postfix"
    seq = "agtc" # DNA sequence to add at end of reads. Checked to be agtcn
    qual = "IIII" # same length as seq. Your responsibility to have valid phred values
    target = "Read1" # Read1|Read2|Index1|Index2
```

This transformation adds a specified sequence and corresponding quality scores to the end of reads.