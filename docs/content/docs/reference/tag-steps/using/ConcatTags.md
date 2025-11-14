---
weight: 50
---

# ConcatTags

Concatenate multiple tags into a single tag.

```toml
[[step]]
    action = "ConcatTags"
    in_labels = ["tag1", "tag2", "tag3"]  # list of tags to concatenate (minimum 2)
    out_label = "combined"  # output tag name
    separator = "_"  # (optional) separator for string concatenation
```

This transformation combines multiple tags into a single output tag. The behavior depends on the types of input tags:

## Behavior by Tag Type

### Location Tags Only
When all input tags are location tags (e.g., from `ExtractIUPAC`, `ExtractAnchor`), the transformation:
- Appends all regions from all tags into a single tag
- Preserves hit ordering (tag1's hits first, then tag2's hits, etc.)
- When displayed (e.g., with `StoreTagInComment`), regions are joined with "_" separator

Example:
```toml
# Extract two barcodes
[[step]]
    action = "ExtractIUPAC"
    search = "AAAA"
    out_label = "barcode1"
    anchor = "Left"

[[step]]
    action = "ExtractIUPAC"
    search = "TTTT"
    out_label = "barcode2"
    anchor = "Right"

# Concatenate them
[[step]]
    action = "ConcatTags"
    in_labels = ["barcode1", "barcode2"]
    out_label = "combined_barcode"
```

If the input sequence is `AAAACGTACGTTTT`, the combined tag will contain both regions and display as `AAAA_TTTT`.

### String Tags Only
When all input tags are string tags (e.g., from `ExtractRegex`), the transformation:
- Concatenates the strings
- Uses the optional `separator` parameter between strings (defaults to no separator)

Example:
```toml
[[step]]
    action = "ConcatTags"
    in_labels = ["prefix", "suffix"]
    out_label = "combined"
    separator = "-"  # strings joined with "-"
```

### Mixed Tag Types
When mixing location and string tags:
- All tags are converted to strings (location tags use their sequences)
- Strings are concatenated with the optional separator
- The result is a string tag (not a location tag)

## Multiple Hits per Tag
If any input tag contains multiple hits (e.g., from `ExtractAnchor` with multiple regions), all hits from all tags are combined:

```toml
# Create a tag with 2 hits
[[step]]
    action = "ExtractAnchor"
    in_label = "anchor1"
    regions = [[0, 2], [2, 2]]  # creates 2 hits
    out_label = "multi_tag1"

# Create another tag with 2 hits
[[step]]
    action = "ExtractAnchor"
    in_label = "anchor2"
    regions = [[0, 2], [2, 2]]
    out_label = "multi_tag2"

# Concatenate both multi-hit tags
[[step]]
    action = "ConcatTags"
    in_labels = ["multi_tag1", "multi_tag2"]
    out_label = "combined"  # will have 4 hits total
```

## Missing Tags
Missing tags are automatically skipped during concatenation. If all input tags are missing for a read, the output tag will also be missing.

## Validation
- Requires at least 2 input tags
- Rejects duplicate input labels
- Validates that all input tags exist before this step in the pipeline
- Does not support Numeric or Bool tags (only Location and String)
