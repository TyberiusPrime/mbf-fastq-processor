---
title: Compare String Tags
---
# CompareStringTags

```toml
[[step]]
    action = "CompareStringTags"
    in_label_a = "tag_a"    # First string or location tag to compare
    in_label_b = "tag_b"    # Second string or location tag to compare
    out_label = "cmp"       # Numeric output: -1, 0, or 1 for smaller, equal, larger
```

Compare two string- or location-valued tags lexicographically (byte-by-byte).

The output tag is numeric:
- `-1` if `tag_a < tag_b`
- `0` if `tag_a == tag_b`
- `1` if `tag_a > tag_b`

Both tags must contain sequences of the same length for every read. 
A runtime error is raised for any read where the two sequences have different lengths.

If either input is `Missing`, the output is `NaN`.

## Comparing extracted sequence regions

```toml
[options]
    accept_duplicate_files = true

[input]
    read1 = 'reads.fq'
    read2 = 'reads.fq'

# Extract a fixed-length barcode from each read
[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 8
    anchor = "Start"
    out_label = "bc1"

[[step]]
    action = "ExtractRegion"
    segment = "read2"
    start = 0
    length = 8
    anchor = "Start"
    out_label = "bc2"

# Compare the two barcodes (-1, 0, or 1)
[[step]]
    action = "CompareStringTags"
    in_label_a = "bc1"
    in_label_b = "bc2"
    out_label = "bc_order"

[[step]]
    action = "StoreTagInComment"
    in_label = "bc_order"

[output]
    prefix = "output"
```

## Combining with EvalExpression

The numeric -1/0/1 output integrates directly with
[EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}}):

```toml
[options]
    accept_duplicate_files = true

[input]
    read1 = 'reads.fq'
    read2 = 'reads.fq'

[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 8
    anchor = "Start"
    out_label = "seq_a"

[[step]]
    action = "ExtractRegion"
    segment = "read2"
    start = 0
    length = 8
    anchor = "Start"
    out_label = "seq_b"

[[step]]
    action = "CompareStringTags"
    in_label_a = "seq_a"
    in_label_b = "seq_b"
    out_label = "cmp"

# Keep only reads where both barcodes are identical
[[step]]
    action = "EvalExpression"
    expression = "cmp == 0"
    result_type = "bool"
    out_label = "seqs_equal"

[[step]]
    action = "FilterByTag"
    in_label = "seqs_equal"
    keep_or_remove = "Keep"

[[step]]
    action = "ForgetAllTags"

[output]
    prefix = "output"
```
