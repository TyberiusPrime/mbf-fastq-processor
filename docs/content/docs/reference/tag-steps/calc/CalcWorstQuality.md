---
weight: 57
---

# CalcWorstQuality

Compute the minimum quality byte value for each read across a segment or a tagged region.

```toml
[[step]]
    action = "CalcWorstQuality"
    out_label = "worst_q"
    source = "read1" # Segment, or location tag
    offset = -33 # Optional offset.
```

The output is the minimum quality across all bases in the segment, all segments
or tag region).

By Default no offset is applied, so you get the minimum ascii value. 
To convert e.g. from Illumina PHRED bytes set `offset = -33`.
