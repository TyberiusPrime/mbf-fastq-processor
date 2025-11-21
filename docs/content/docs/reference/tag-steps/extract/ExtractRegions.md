---
weight: 50
---

# ExtractRegions

Extract from multiple regions with flexible source and anchoring options.

## Basic Usage

Extract from fixed positions in sequence data:

```toml
[[step]]
    action = "ExtractRegions"
    regions = [
        {source = "read1", start = 0, length = 8, anchor= "Start"},
        {source = "read1", start = -12, length = 4, anchor="End"},
    ]
    out_label = "barcode"
```

## Advanced Usage with Sources

Extract from tag-derived positions:

```toml
# First create an anchor tag
[[step]]
    action = "ExtractIUPAC"
    search = "CAYA"
    out_label = "anchor_tag"
    segment = "read1"
    anchor = "Anywhere"
    max_mismatches = 0

# Then extract relative to that anchor
[[step]]
    action = "ExtractRegions"
    regions = [
        { source = "tag:anchor_tag", start = -2, length = 4, anchor = "Start" },
        { source = "tag:anchor_tag", start = 4, length = 1, anchor = "Start" }
    ]
    out_label = "relative_regions"
```

Extract from read names:

```toml
[[step]]
    action = "ExtractRegions"
    regions = [
        { source = "name:read1", start = 0, length = 10, anchor="Start" }
    ]
    out_label = "name_prefix"
```

## Region Parameters

- `segment`: Source segment (legacy, for fixed positions)
- `source`: Flexible source specification:
  - `"segment_name"` or `"read1"` - extract from segment sequences
  - `"tag:tag_name"` - extract from tag-derived positions (see below)
  - `"name:segment"` - extract from read names
- `start`: Start position (can be negative with anchoring)
- `length`: Length of region to extract
- `anchor`: Anchoring mode (optional, default: "Start")
  - `"Start"` - position relative to sequence start
  - `"End"` - position relative to sequence end

## Tag specifics

For location tags, the extracted sequence is from the underlying source (if still available),
that is from the original read sequence (not the possibly altered stored sequence).

For string tags, the extracted sequence is from the tag string itself.


## Regions outside of the sequence

Regions that start before the sequence or extend beyond the sequence will 
lead to non-matching / missing tags.


## Notes

- Start positions are 0-based
- ExtractRegions with only one region is equivalent to [ExtractRegion]({{< relref "docs/reference/tag-steps/extract/ExtractRegion.md" >}})
- If the read is shorter than requested, the region will be shorter
- This transformation replaces the functionality of the deprecated ExtractAnchor
- When using `source = "tag:..."`, the tag must provide location information
