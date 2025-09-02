---
weight: 50
---

# ExtractAnchor

Extract regions relative to an anchor sequence.

```toml
[[step]]
    action = "ExtractAnchor"
    label = "mytag"
    search = "CYCTT" # IUPAC pattern to search for
    target = "Read1" # Read1|Read2|Index1|Index2
    regions = [[-2, 4], [4, 1]] # [start, length] pairs relative to anchor
    region_separator = "_" # (optional) separator between regions
```

This transformation searches for an anchor sequence using IUPAC pattern matching and extracts specified regions relative to the anchor's position. The regions are defined as [start, length] pairs where start is relative to the leftmost position of the anchor match (can be negative). Multiple regions are concatenated with the specified separator.