# Tag / Label

A tag is a piece of fragment-derived metadata that one step in the pipeline produces, and other steps may consume, transform, or export.

## Overview

Tags enable sophisticated workflows by decoupling data extraction from data usage. Instead of hardcoding logic like "trim adapters AND filter by adapter presence" into a single step, you extract adapter locations as a tag, then use that tag in multiple downstream operations.

Tags are identified by labels (arbitrary names following the pattern `[a-zA-Z_][a-zA-Z0-9_]*`) and carry typed values that describe properties of each fragment.

## Tag Types

mbf-fastq-processor supports four tag types:

### Location+Sequence Tags

Represent a region within a [segment]({{< relref "docs/concepts/segments.md" >}}), storing:
- Start position (0-based, inclusive)
- End position (0-based, exclusive)
- The extracted sequence
- Optionally, the segment name

**Created by:**
- [ExtractIUPAC]({{< relref "docs/reference/tag-steps/extract/ExtractIUPAC.md" >}}) – Find IUPAC patterns (e.g., adapters, barcodes)
- [ExtractRegex]({{< relref "docs/reference/tag-steps/extract/ExtractRegex.md" >}}) – Find regex patterns
- [ExtractRange]({{< relref "docs/reference/tag-steps/extract/ExtractRange.md" >}}) – Extract fixed coordinate ranges

**Used by:**
- [TrimAtTag]({{< relref "docs/reference/modification-steps/TrimAtTag.md" >}}) – Cut segment at tag location
- [LowercaseTag]({{< relref "docs/reference/modification-steps/LowercaseTag.md" >}}) – Lowercase the tagged region
- [UppercaseTag]({{< relref "docs/reference/modification-steps/UppercaseTag.md" >}}) – Uppercase the tagged region
- [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}) – Keep/remove fragments based on tag presence
- [ExtractToName]({{< relref "docs/reference/modification-steps/ExtractToName.md" >}}) – Append tag sequence to read name

**Note:** Location tags may lose their location information when transformed by certain steps, becoming sequence-only tags.

### Sequence-Only Tags

Store just a sequence string without positional information.

**Created by:**
- Transforming location tags (some steps strip location data)
- [ConvertTagToSequence]({{< relref "docs/reference/tag-steps/convert/ConvertTagToSequence.md" >}}) – Explicitly convert location tags

**Used by:**
- [FilterBySequenceTag]({{< relref "docs/reference/filter-steps/FilterBySequenceTag.md" >}}) – Filter based on sequence content
- [ExtractToName]({{< relref "docs/reference/modification-steps/ExtractToName.md" >}}) – Add sequence to read name
- [StoreTagsInTable]({{< relref "docs/reference/tag-steps/using/StoreTagsInTable.md" >}}) – Export to TSV

### Numeric Tags

Store floating-point or integer values representing computed metrics.

**Created by:**
- [CalcMeanQuality]({{< relref "docs/reference/tag-steps/calc/CalcMeanQuality.md" >}}) – Average quality score
- [CalcGCContent]({{< relref "docs/reference/tag-steps/calc/CalcGCContent.md" >}}) – GC percentage
- [CalcLength]({{< relref "docs/reference/tag-steps/calc/CalcLength.md" >}}) – Sequence length
- [EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}}) – Compute from other tags
- [ConvertTagToNumeric]({{< relref "docs/reference/tag-steps/convert/ConvertTagToNumeric.md" >}}) – Parse numeric from string tags

**Used by:**
- [FilterByNumericTag]({{< relref "docs/reference/filter-steps/FilterByNumericTag.md" >}}) – Threshold filtering
- [QuantifyTag]({{< relref "docs/reference/report-steps/QuantifyTag.md" >}}) – Generate histograms and statistics
- [EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}}) – Combine in calculations
- [StoreTagsInTable]({{< relref "docs/reference/tag-steps/using/StoreTagsInTable.md" >}}) – Export to TSV

### Boolean Tags

Store true/false flags indicating fragment properties.

**Created by:**
- [TagIfTagPresent]({{< relref "docs/reference/tag-steps/tag/TagIfTagPresent.md" >}}) – Tag based on other tag existence
- [TagByLength]({{< relref "docs/reference/tag-steps/tag/TagByLength.md" >}}) – Tag based on segment length
- [TagByNumericTag]({{< relref "docs/reference/tag-steps/tag/TagByNumericTag.md" >}}) – Tag based on numeric threshold

**Used by:**
- [FilterByBooleanTag]({{< relref "docs/reference/filter-steps/FilterByBooleanTag.md" >}}) – Keep/remove flagged fragments
- [StoreTagsInTable]({{< relref "docs/reference/tag-steps/using/StoreTagsInTable.md" >}}) – Export flags

## Tag Lifecycle

Tags follow a strict lifecycle enforced by the processor:

1. **Definition**: A step with `out_label` creates a tag
2. **Consumption**: Steps with `in_label` or `in_labels` read the tag
3. **Transformation**: Convert steps modify tags into new tags
4. **Removal**: Consuming steps may delete tags (e.g., `ForgetTag`)

**Validation:** At startup, the processor verifies:
- Every tag is defined before use
- Every defined tag is eventually consumed
- Tag names follow the naming rules

This catches typos (e.g., `in_label = "adaptor"` when you created `out_label = "adapter"`) before processing begins.

## Tag Workflow Patterns

### Pattern 1: Extract-Then-Trim

Find adapters, then remove them:

```toml
[[step]]
name = "ExtractIUPAC"
segment = "read1"
pattern = "AGATCGGAAGAGC"
out_label = "adapter"

[[step]]
name = "TrimAtTag"
segment = "read1"
in_label = "adapter"
trim_side = "RightOfMatch"
```

### Pattern 2: Extract-Filter-Export

Find barcodes, keep matches, export sequences:

```toml
[[step]]
name = "ExtractIUPAC"
segment = "index1"
pattern = "NNNATCACGNNNN"     # 4bp + barcode + 4bp
out_label = "full_index"

[[step]]
name = "FilterByTag"
in_label = "full_index"
action = "Keep"                # Remove fragments without match

[[step]]
name = "ExtractRange"
segment = "index1"
start = 4
end = 10
out_label = "barcode"

[[step]]
name = "StoreTagsInTable"
in_labels = ["barcode"]
filename = "barcodes.tsv"
```

### Pattern 3: Calculate-Threshold-Report

Compute quality, filter, then report distribution:

```toml
[[step]]
name = "CalcMeanQuality"
segment = "read1"
out_label = "mean_q"

[[step]]
name = "Report"                # Before filtering

[[step]]
name = "FilterByNumericTag"
in_label = "mean_q"
minimum = 30

[[step]]
name = "Report"                # After filtering

[[step]]
name = "QuantifyTag"
in_label = "mean_q"
filename = "quality_histogram.json"
```

### Pattern 4: Multi-Tag Boolean Logic

Combine multiple tags with expression evaluation:

```toml
[[step]]
name = "CalcLength"
segment = "read1"
out_label = "len_r1"

[[step]]
name = "CalcMeanQuality"
segment = "read1"
out_label = "q_r1"

[[step]]
name = "EvalExpression"
expression = "len_r1 >= 100 && q_r1 >= 30"
out_label = "high_quality"

[[step]]
name = "FilterByNumericTag"
in_label = "high_quality"
minimum = 1                    # Keep only true (1) values
```

### Pattern 5: Tag Reuse

Use one tag for multiple purposes:

```toml
[[step]]
name = "ExtractIUPAC"
segment = "read1"
pattern = "AGATCGGAAGAGC"
out_label = "adapter"

[[step]]
name = "TagIfTagPresent"
in_label = "adapter"
out_label = "has_adapter"      # Boolean flag

[[step]]
name = "LowercaseTag"
segment = "read1"
in_label = "adapter"           # Mask adapter in sequence

[[step]]
name = "ExtractToName"
segment = "read1"
in_label = "adapter"           # Add adapter to read name
with_position = true

[[step]]
name = "StoreTagsInTable"
in_labels = ["has_adapter", "adapter"]
filename = "adapters.tsv"
```

## Tag Naming Rules

Tag labels must:
- Match the regex `[a-zA-Z_][a-zA-Z0-9_]*`
- Be case-sensitive (`mean_q` ≠ `Mean_Q`)
- Not be `ReadName` (reserved for table output)
- Not start with `len_` (reserved for virtual tags in [EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}}))

**Good names:**
- `adapter_r1`
- `barcode_fwd`
- `mean_quality_passing`
- `gc_content`

**Invalid names:**
- `mean-quality` (contains hyphen)
- `2adapter` (starts with number)
- `ReadName` (reserved)
- `len_adapter` (reserved prefix)

## Advanced Usage

### Virtual Tags in EvalExpression

When using [EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}}), you can reference tag lengths with `len_<tagname>`:

```toml
[[step]]
name = "ExtractIUPAC"
segment = "read1"
pattern = "NNNN"
out_label = "umi"

[[step]]
name = "EvalExpression"
expression = "len_umi == 4"    # Virtual tag: length of UMI
out_label = "correct_umi_length"
```

### Tag Chaining

Transform tags through multiple steps:

```toml
[[step]]
name = "ExtractRange"
segment = "index1"
start = 0
end = 8
out_label = "index_raw"        # Location tag

[[step]]
name = "ConvertTagToSequence"
in_label = "index_raw"
out_label = "index_seq"        # Sequence tag

[[step]]
name = "ReverseComplement"
in_label = "index_seq"
out_label = "index_rc"         # Transformed sequence

[[step]]
name = "ExtractToName"
in_label = "index_rc"
```

### Conditional Processing

Tags enable conditional logic without branching:

```toml
# Tag long reads
[[step]]
name = "TagByLength"
segment = "read1"
minimum = 200
out_label = "is_long"

# Filter differently based on tag (via boolean conversion)
[[step]]
name = "FilterByBooleanTag"
in_label = "is_long"
keep_true = true
```

## See Also

- [Tag extraction reference]({{< relref "docs/reference/tag-steps/_index.md" >}}) for all tag-generating steps
- [Step concept]({{< relref "docs/concepts/step.md" >}}) for tag lifecycle validation
- [Source concept]({{< relref "docs/concepts/source.md" >}}) for using tags as data sources
