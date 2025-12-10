# Source

When a step refers to a 'source' (instead of a [`segment`]({{< relref "docs/concepts/segments.md" >}})), it means the step can read from multiple types of data: segment sequences, segment names, or tag values.

## Overview

The `source` parameter generalizes the `segment` parameter, allowing steps to operate on different kinds of string data within a fragment. This flexibility enables advanced workflows like extracting patterns from read names, processing tag-derived sequences, or combining multiple data sources.

## Source Types

### Segment Source

Reads the sequence from a specific segment.

**Syntax:** Just the segment name (e.g., `"read1"`, `"index1"`)

**Example:**
```toml
[[step]]
    action = "ExtractIUPAC"
    segment = "read1"              # Read from read1 segment sequence
    search = "AGATCGGAAGAGC"
    anchor = "anywhere"
    out_label = "adapter"
    max_mismatches = 1
```

This is functionally identical to using `segment = "read1"` in steps that support both parameters.

### Name Source

Reads the read name (FASTQ header) from a specific segment.

**Syntax:** `"name:<segment>"` (e.g., `"name:read1"`, `"name:index1"`)

**Example:**
```toml
[[step]]
    action = "ExtractRegex"
    source = "name:read1"         # Read from read1's name/header
    pattern = "BC:([ACGT]+)"      # Extract barcode from name
    out_label = "barcode_from_name"
```

**Use cases:**
- Extracting barcodes embedded in read names by upstream tools
- Parsing instrument-specific metadata from headers
- Extracting UMIs added to read names during library prep

### Tag Source

Reads the sequence value from a previously defined tag.

**Syntax:** `"tag:<label>"` (e.g., `"tag:barcode"`, `"tag:adapter"`)

**Requirements:**
- The tag must be a location tag or sequence tag
- The tag must be defined earlier in the pipeline

**Example:**
```toml
[[step]]
    action = "ExtractIUPAC"
    segment = "read1"    # Search within the extracted barcode
    max_mismatches = 0
    search = "NNNATCG"
    out_label = "hit"
    anchor = "anywhere"

[[step]]
    action = "ExtractRegion"
    source = "tag:hit"
    start = 0
    length = 3
    anchor = "Start"
    out_label = "barcode"     # Extract first 3bp

```

**Use cases:**
- Searching within previously extracted regions
- Processing tag-derived sequences without affecting original segments
- Multi-stage pattern matching (extract large region, then search within it)

## Segment vs Source

Many steps accept either `segment` or `source`, but not both:

| Parameter | Accepts | Example Values |
|-----------|---------|----------------|
| `segment` | Segment names only | `"read1"`, `"index1"`, `"All"` |
| `source` | Segments, names, or tags | `"read1"`, `"name:read1"`, `"tag:barcode"` |

**When to use `segment`:**
- Simple operations on segment sequences
- Steps that modify segment data in place
- When you only work with segment sequences

**When to use `source`:**
- Extracting from read names
- Processing tag-derived sequences
- Flexible workflows that might switch data sources

## Supported Steps

Not all steps support `source`. Common steps that do:

- [ExtractRegex]({{< relref "docs/reference/tag-steps/extract/ExtractRegex.md" >}})
- [TagDuplicates]({{< relref "docs/reference/tag-steps/tag/TagDuplicates.md" >}})
- [ValidateAllReadsSameLength]({{< relref "docs/reference/validation-steps/ValidateAllReadsSameLength.md" >}})


## Limitations

- Tag sources must be location or sequence tags (not numeric or boolean)
- Name sources access the entire FASTQ header line (excluding the leading `@`)

## See Also

- [Segments concept]({{< relref "docs/concepts/segments.md" >}}) for understanding segment basics
- [Tag concept]({{< relref "docs/concepts/tag.md" >}}) for tag types and workflows
- [Reference documentation]({{< relref "docs/reference/_index.md" >}}) for step-specific source support
