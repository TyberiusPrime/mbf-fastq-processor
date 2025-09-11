---
weight: 50
---

# ExtractAnchor

Extract regions relative to a previously tagged anchor position.

```toml
# First create an anchor tag. Iupac, regex, ExtractRegion, your choice.
[[step]]
    action = "ExtractIUPAC"
    search = "CAYA"
    label = "anchor_tag"
    segment = "read1"
    anchor = "Anywhere"
    max_mismatches = 0

# Then extract relative to that anchor
[[step]]
    action = "ExtractAnchor"
    label = "mytag"
    input_label = "anchor_tag" # tag that provides the anchor position
    regions = [[-2, 4], [4, 1]] # [start, length] pairs relative to anchor
    region_separator = "_" # (optional) separator between regions
```

This transformation uses the leftmost position of a previously established tag as the anchor point and extracts specified regions relative to that position.

The regions are defined as [start, length] pairs where start is relative to the leftmost position of the referenced tag (can be negative). 

Multiple regions are concatenated with the specified separator.

Note: This transformation requires a tag that provides location information (such as those created by ExtractIUPAC, ExtractRegex, or ExtractRegion(s).
