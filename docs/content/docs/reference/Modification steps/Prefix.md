---
weight: 50
---

# Prefix

Add text to the beginning of read sequences.

```toml
[[step]]
    action = "Prefix"
    seq = "agtTCAa" # DNA sequence to add at beginning of reads. Checked to be agtcn
    qual = "IIIBIII" # same length as seq. Your responsibility to have valid phred values
    segment = "read1" # Any of your input segments
```

This transformation adds a specified sequence and corresponding quality scores to the beginning of reads.