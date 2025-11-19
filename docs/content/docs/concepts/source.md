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
name = "ExtractIUPAC"
source = "read1"              # Read from read1 segment sequence
pattern = "AGATCGGAAGAGC"
out_label = "adapter"
```

This is functionally identical to using `segment = "read1"` in steps that support both parameters.

### Name Source

Reads the read name (FASTQ header) from a specific segment.

**Syntax:** `"name:<segment>"` (e.g., `"name:read1"`, `"name:index1"`)

**Example:**
```toml
[[step]]
name = "ExtractRegex"
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
name = "ExtractRange"
segment = "index1"
start = 0
end = 8
out_label = "raw_barcode"     # Extract first 8bp

[[step]]
name = "ExtractIUPAC"
source = "tag:raw_barcode"    # Search within the extracted barcode
pattern = "NNNATCG"
out_label = "barcode_motif"
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

## Practical Examples

### Example 1: UMI in Read Name

Extract a UMI (Unique Molecular Identifier) from the read name:

```toml
[input]
read1 = ["sample_R1.fq.gz"]
read2 = ["sample_R2.fq.gz"]

# Read names look like: @M00123:123:000000000-A1234:1:1101:15000:1234:UMI:ACGTACGT
[[step]]
name = "ExtractRegex"
source = "name:read1"
pattern = "UMI:([ACGT]{8})"
out_label = "umi"

[[step]]
name = "ExtractToName"
in_label = "umi"
segment = "All"
separator = "_"
# Now UMI is also in the output read name for downstream tools
```

### Example 2: Nested Tag Extraction

Extract a region, then search within it:

```toml
# Extract a large region that might contain adapters
[[step]]
name = "ExtractRange"
segment = "read1"
start = 80
end = 120
out_label = "tail_region"

# Search for adapter within that region
[[step]]
name = "ExtractIUPAC"
source = "tag:tail_region"
pattern = "AGATCGGAAGAGC"
out_label = "adapter_in_tail"

# Filter: keep only reads with adapter in the tail region
[[step]]
name = "FilterByTag"
in_label = "adapter_in_tail"
action = "Keep"
```

### Example 3: Barcode Validation

Extract barcode from index, validate its structure:

```toml
[[step]]
name = "ExtractRange"
segment = "index1"
start = 0
end = 8
out_label = "barcode_raw"

# Check that the barcode doesn't contain Ns
[[step]]
name = "ExtractRegex"
source = "tag:barcode_raw"
pattern = "^[ACGT]{8}$"       # Only ACGT, no Ns
out_label = "barcode_valid"

[[step]]
name = "FilterByTag"
in_label = "barcode_valid"
action = "Keep"               # Discard fragments with Ns in barcode

# Now safe to use the barcode
[[step]]
name = "ExtractToName"
in_label = "barcode_raw"
segment = "All"
```

### Example 4: Sample ID from Read Name

Parse sample ID embedded in read names by a demultiplexer:

```toml
# Read names: @M00123:123:000000000-A1234:1:1101:15000:1234 SAMPLE:Sample01
[[step]]
name = "ExtractRegex"
source = "name:read1"
pattern = "SAMPLE:([^ ]+)"
out_label = "sample_id"

[[step]]
name = "StoreTagsInTable"
in_labels = ["sample_id"]
filename = "sample_assignments.tsv"
```

### Example 5: Complex Adapter Detection

Search multiple potential adapter locations:

```toml
# Check for adapter in full read
[[step]]
name = "ExtractIUPAC"
source = "read1"
pattern = "AGATCGGAAGAGC"
out_label = "adapter_full"

# Also check last 40bp specifically
[[step]]
name = "ExtractRange"
segment = "read1"
start = -40
end = -1
out_label = "read_tail"

[[step]]
name = "ExtractIUPAC"
source = "tag:read_tail"
pattern = "AGATCGGAAGAGC"
out_label = "adapter_tail"

# Tag if adapter found anywhere
[[step]]
name = "TagIfTagPresent"
in_label = "adapter_full"
out_label = "has_adapter"

[[step]]
name = "TagIfTagPresent"
in_label = "adapter_tail"
out_label = "has_adapter_tail"

# Report statistics
[[step]]
name = "StoreTagsInTable"
in_labels = ["has_adapter", "has_adapter_tail"]
filename = "adapter_detection.tsv"
```

## Supported Steps

Not all steps support `source`. Common steps that do:

### Extraction Steps
- [ExtractIUPAC]({{< relref "docs/reference/tag-steps/extract/ExtractIUPAC.md" >}})
- [ExtractRegex]({{< relref "docs/reference/tag-steps/extract/ExtractRegex.md" >}})
- [ExtractKmers]({{< relref "docs/reference/tag-steps/extract/ExtractKmers.md" >}})

### Calculation Steps
- [CalcLength]({{< relref "docs/reference/tag-steps/calc/CalcLength.md" >}})
- [CalcGCContent]({{< relref "docs/reference/tag-steps/calc/CalcGCContent.md" >}})

Check the [reference documentation]({{< relref "docs/reference/_index.md" >}}) for each step to see if it supports `source`.

## Limitations

- Tag sources must be location or sequence tags (not numeric or boolean)
- Name sources read the entire FASTQ header line (excluding the leading `@`)
- Tag sources lose positional information from location tags when used as sources

## See Also

- [Segments concept]({{< relref "docs/concepts/segments.md" >}}) for understanding segment basics
- [Tag concept]({{< relref "docs/concepts/tag.md" >}}) for tag types and workflows
- [Reference documentation]({{< relref "docs/reference/_index.md" >}}) for step-specific source support
