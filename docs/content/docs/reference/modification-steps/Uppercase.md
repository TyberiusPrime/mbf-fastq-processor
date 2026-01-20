
weight: 150
---

# Uppercase

Convert sequences, tags, or read names to uppercase.

```toml
[[step]]
    action = "Uppercase"
    target = "read1"  # Any input segment, 'All', 'tag:mytag', or 'name:read1'
    #if_tag = "mytag"  # Optional: only apply if tag is truthy
```

## Target Options

- **Segment**: `"read1"`, `"read2"`, `"index1"`, `"index2"`, or `"All"` - lowercase's sequence
- **Tag**: `"tag:mytag"` - lowercase's tag's sequence content (Location-type tags only)
- **Name**: `"name:read1"` - lowercase's read name (not including comments)

Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy via `if_tag`.

## Examples

### Uppercase a segment
```toml
[[step]]
    action = "Uppercase"
    target = "read1"
```

### Uppercase a tag
```toml
[[step]]
    action = "ExtractIUPAC"
    segment = 'read1'
    search = "CTN"
    out_label = "mytag"
    anchor = "Left"
    max_mismatches = 0

[[step]]
    action = "Uppercase"
    target = "tag:mytag"
```

Follow with [StoreTagInSequence]({{< relref "docs/reference/tag-steps/using/StoreTagInSequence.md" >}}) to apply lowercase tag back onto the read.

### Uppercase read names
```toml
[[step]]
    action = "Uppercase"
    target = "name:read1"
```

### Conditional uppercase
```toml
[[step]]
    action = "ExtractIUPAC"
    segment = 'read1'
    search = "CTN"
    out_label = "had_iupac"
    anchor = "Left"
    max_mismatches = 0

[[step]]
    action = "Uppercase"
    target = "read1"
    if_tag = "had_iupac"
```
