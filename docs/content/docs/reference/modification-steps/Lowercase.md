
weight: 150
---

# Lowercase

Convert sequences, tags, or read names to lowercase.

```toml
[[step]]
    action = "Lowercase"
    target = "read1"  # Any input segment, 'All', 'tag:mytag', or 'name:read1'
    #if_tag = "mytag"  # Optional: only apply if tag is truthy
```

## Target Options

- **Segment**: `"read1"`, `"read2"`, `"index1"`, `"index2"`, or `"All"` - lowercase's sequence
- **Tag**: `"tag:mytag"` - lowercase's tag's sequence content (Location-type tags only)
- **Name**: `"name:read1"` - lowercase's read name (not including comments)

Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy via `if_tag`.

## Examples

### Lowercase a segment
```toml
[[step]]
    action = "Lowercase"
    target = "read1"
```

### Lowercase a tag
```toml
[[step]]
    action = "ExtractIUPAC"
    segment = 'read1'
    search = "CTN"
    out_label = "mytag"
    anchor = "Left"
    max_mismatches = 0

[[step]]
    action = "Lowercase"
    target = "tag:mytag"
```

Follow with [StoreTagBackInSequence]({{< relref "docs/reference/tag-steps/using/StoreTagBackInSequence.md" >}}) to apply lowercase tag back onto the read.

### Lowercase read names
```toml
[[step]]
    action = "Lowercase"
    target = "name:read1"
```

### Conditional lowercase
```toml
[[step]]
    action = "ExtractIUPAC"
    segment = 'read1'
    search = "CTN"
    out_label = "had_iupac"
    anchor = "Left"
    max_mismatches = 0

[[step]]
    action = "Lowercase"
    target = "read1"
    if_tag = "had_iupac"
```