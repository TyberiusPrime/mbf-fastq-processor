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
    if_tag = "mytag"
    encoding = 'Illumina1.8' #  optional, default=sanger 'Illumina1.8|Illumina1.3|Sanger|Solexa'
    # Illumina1.8 is an alias for Sanger.
```

This transformation adds a specified sequence and corresponding quality scores to the beginning of reads.

Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy.

Quality characters must be in the range defined by the encoding.
