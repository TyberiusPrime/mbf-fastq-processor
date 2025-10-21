---
weight: 50
---

# ReplaceTagWithLetter

Replace sequence bases in tagged regions with a specified letter.

```toml
[[step]]
    action = "ReplaceTagWithLetter"
    label = "mytag"  # Tag containing regions to replace
    letter = "N"   # Replacement character (defaults to 'N')
```

This transformation replaces all sequence bases within the regions defined by a tag with a specified replacement character. Quality scores are preserved unchanged. This is commonly used to mask low-quality regions as 'N' characters.

## Parameters

- `label`: Name of the tag containing regions to be replaced
- `letter`: Single character to replace bases with (defaults to 'N' if not specified)

## Example Use Cases

- Mask low-quality bases identified by [ExtractRegionsOfLowQuality]({{< relref "docs/reference/tag-steps/extract/ExtractRegionsOfLowQuality.md" >}})
- Replace specific sequence motifs identified by other extraction steps
- Convert tagged regions to ambiguous bases for downstream analysis

The tag must have been created by a previous extraction step and must contain location information.
