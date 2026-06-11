---
weight: 50
---

# ConcatTags

Concatenate multiple tags into a single tag.

```toml
# ignore_in_test
[[step]]
    action = "ConcatTags"
    in_labels = ["mytag", "mytag2"]  # list of tags to concatenate (minimum 2)
    out_label = "combined"  # output tag name
    on_missing = "merge_present"  # required: "merge_present" or "set_missing"
    separator = "_"  # (optional) separator for string concatenation
```

This transformation combines multiple tags into a single (string) output tag. 

Briefly it 
- Concatenates the strings
- Uses the optional `separator` parameter between strings (defaults to no separator)

Example:
```toml
# ignore_in_test
[[step]]
    action = "ConcatTags"
    in_labels = ["prefix", "suffix"]
    out_label = "combined"
    on_missing = "merge_present"
    separator = "-"  # strings joined with "-"
```

### Mixed 
## Multiple Hits per Tag
If any input tag contains multiple hits (e.g., from `ExtractAnchor` with multiple regions), 
all hits from all tags are combined, and the separater is placed between each one.

(You can fake 'location separators' by first ConcatTags on a single location tag, then
ConcatTags on the combined string tags).

```toml
# ignore_in_test
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
    on_missing = "merge_present"
```

## Handling Missing Tags

The `on_missing` parameter controls how the transformation handles reads where some input tags are missing:

### `merge_present`
Skips missing tags and concatenates only the present ones:
```toml
# ignore_in_test
[[step]]
    action = "ConcatTags"
    in_labels = ["barcode1", "barcode2"]
    out_label = "combined"
    on_missing = "merge_present"
```

Behavior:
- If both tags present: `combined = barcode1 + barcode2`
- If only barcode1 present: `combined = barcode1`
- If only barcode2 present: `combined = barcode2`
- If neither present: `combined` is missing

### `set_missing`
Sets the output tag to missing if any input tag is missing:
```toml
# ignore_in_test
[[step]]
    action = "ConcatTags"
    in_labels = ["barcode1", "barcode2"]
    out_label = "combined"
    on_missing = "set_missing"
```

Behavior:
- If both tags present: `combined = barcode1 + barcode2`
- If any tag missing: `combined` is missing

Use `set_missing` when you need complete information from all tags, and use `merge_present` when partial information is acceptable.

## Validation
- Requires at least 2 input tags
- Rejects duplicate input labels
- Validates that all input tags exist before this step in the pipeline
- Does not support Numeric or Bool tags (only Location and String)
