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
    segment = "read1" # Any of your input segments
    if_tag = "mytag"
    encoding = 'Illumina1.8' #  optional, default=sanger 'Illumina1.8|Illumina1.3|Sanger|Solexa'
    # Illumina1.8 is an alias for Sanger.
```

This transformation adds a specified sequence and corresponding quality scores to the end of reads.

Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy.

Quality characters must be in the range defined by the encoding.
