# Tag / Label

A regular tag is a piece of fragment-derived metadata that one step in the pipeline
produces, and other steps may consume, transform, or export.

A virtual tag is an on-the-fly create tag that exists just
for this step and disappears right afterwards.

## Overview - Regular tags

Tags enable sophisticated workflows by decoupling data extraction from data
usage. Instead of hardcoding logic like "trim adapters AND filter by adapter
presence" into a single step, you extract adapter locations as a tag, then use
that tag in multiple downstream operations.

Tags are identified by labels (arbitrary names following the pattern
`[a-zA-Z_][a-zA-Z0-9_]*`) and carry typed values that describe properties of each
fragment.

Virtual tags are identified by having a specif 'xyz_' prefix. See [below](#virtual_tags).
You can not declare a tag with that prefix as out_label of any step.

(The tag 'ReadName' is also reserved for usage in StoreTagsInTable's index column)

## Tag Types

fastqrab supports four tag types:

(None of the subsequent step listings below are exhaustive).

### Location+Sequence Tags

Represent a region within a [segment]({{< relref "docs/concepts/segments.md" >}}), storing:
- A segment reference,
- Start position (0-based, inclusive)
- End position (0-based, exclusive)
- The extracted sequence (which may be changed by downstream steps) 

If you modify the segment's sequence, tag positions may become invalid.
The extracted sequence however is retained. 

**Created by:**
- [ExtractIUPAC]({{< relref "docs/reference/tag-steps/extract/ExtractIUPAC.md" >}}) – Find IUPAC patterns (e.g., adapters, barcodes)
- [ExtractRegex]({{< relref "docs/reference/tag-steps/extract/ExtractRegex.md" >}}) – Find regex patterns
- [ExtractRegion]({{< relref "docs/reference/tag-steps/extract/ExtractRegion.md" >}}) – Extract fixed coordinate regions

**Used for example by:**
- [TrimAtTag]({{< relref "docs/reference/modification-steps/TrimAtTag.md" >}}) – Cut segment at tag location
- [Lowercase]({{< relref "docs/reference/modification-steps/Lowercase.md" >}}) – Lowercase sequences, tags, or names
- [Uppercase]({{< relref "docs/reference/modification-steps/Uppercase.md" >}}) – Uppercase the stored sequence using 'target="tag:..."' (follow with [StoreTagBackInSequence]({{< relref "docs/reference/tag-steps/using/StoreTagBackInSequence.md" >}}) )
- [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}) – Keep/remove fragments based on tag presence
- [QuantifyTag]({{< relref "docs/reference/report-steps/QuantifyTag.md" >}}) – Generate histograms and statistics
-  [StoreTagInComment]({{< relref "docs/reference/tag-steps/using/StoreTagInComment.md" >}}) – Append tag sequence to read name
- [StoreTagsInTable]({{< relref "docs/reference/tag-steps/using/StoreTagsInTable.md" >}}) – Export to TSV

### Sequence-Only Tags

Store just a sequence string without positional information.

**Created by:**
-  [ExtractRegex]({{< relref "docs/reference/tag-steps/extract/ExtractRegex.md" >}}) with a name or tag source.

**Used by:**
- [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}})
-  [StoreTagInComment]({{< relref "docs/reference/tag-steps/using/StoreTagInComment.md" >}}) – Append tag sequence to read name
- [StoreTagsInTable]({{< relref "docs/reference/tag-steps/using/StoreTagsInTable.md" >}}) – Export to TSV

### Numeric Tags

Store floating-point or integer values representing computed metrics.

Some steps declare ranges on the tag (lower..=upper, left & right inclusive, e.g. 
- [CalcGCContent]({{< relref "docs/reference/tag-steps/calc/CalcGCContent.md" >}}) declares 0..=1 
if relative=true). The thresholds in FilterbyNumericTag are then checked against these limits.

**Created by:**
- [CalcMeanQuality]({{< relref "docs/reference/tag-steps/calc/CalcMeanQuality.md" >}}) – Average quality score
- [CalcGCContent]({{< relref "docs/reference/tag-steps/calc/CalcGCContent.md" >}}) – GC percentage
- [CalcLength]({{< relref "docs/reference/tag-steps/calc/CalcLength.md" >}}) – Sequence length
- [EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}}) – Compute from other tags (if `return_type` == 'numeric')

**Used by:**
- [FilterByNumericTag]({{< relref "docs/reference/filter-steps/FilterByNumericTag.md" >}}) – Threshold filtering
- [EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}}) – Combine in calculations
- [StoreTagsInTable]({{< relref "docs/reference/tag-steps/using/StoreTagsInTable.md" >}}) – Export to TSV

### Boolean Tags

Store true/false flags indicating fragment properties.

**Created by:**
- [EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}})
- [TagDuplicates]({{< relref "docs/reference/tag-steps/tag/TagDuplicates.md" >}})
- [TagOtherFile]({{< relref "docs/reference/tag-steps/tag/TagOtherFile.md" >}})


**Used by:**
- [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}})
- [StoreTagsInTable]({{< relref "docs/reference/tag-steps/using/StoreTagsInTable.md" >}}) – Export flags

## Tag Lifecycle

Tags follow a strict life-cycle enforced by the processor:

1. **Definition**: A step with `out_label` creates a tag
2. **Consumption**: Steps with `in_label` or `in_labels` read the tag
3. **Transformation**: Convert steps modify tags into new tags
4. **Removal**: Consuming steps may delete tags (e.g., `ForgetTag`)

**Validation:** At startup, the processor verifies:
- Every tag is defined before use
- Every defined tag is eventually consumed
- The types of consumed tags match step expectations
- Tag names follow the naming rules

This catches typos (e.g., `in_label = "adaptor"` when you created `out_label = "adapter"`) before processing begins.
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

## Virtual tags


Any place you can use a tag, you can also use virtual tags.

The following virtual tags are supported:

* read_no - the sequential number of the molecule in the input.
* len_<segment|all> - the length of the read (or the molecule) at this step in the pipeline. 
* len_<tag_name> - the length of a tag's string value (for location tags, that's after regex replacement etc).
  (requires a string or location typed tag)
* location_<tag_name> - the location of a (location) tag, as string typed segment:start..end (left inclusive, right exclusive, 0 based)

### Example Len Tags in EvalExpression

When using [EvalExpression]({{< relref
"docs/reference/tag-steps/convert/EvalExpression.md" >}}), you can reference
tag lengths with `len_<tagname>`:

```toml
[[step]]
    action = "ExtractIUPAC"
    segment = "read1"
    search = "NNNN"
    anchor = "anywhere"
    max_mismatches = 0
    out_label = "umi"

[[step]]
    action = "EvalExpression"
    expression = "len_umi == 4"    # Virtual tag: length of UMI
    out_label = "correct_umi_length"
    result_type = 'bool'
```

### Conditional Processing

Modifying tags can be applied conditionally:

```toml
# Tag long reads
[[step]]
    action = "EvalExpression"
    expression = "len_read1 < 100" 
    out_label = "is_short"
    result_type = 'bool'

# Filter differently based on tag (via boolean conversion)
[[step]]
    action = "Postfix"
    seq = "AGGGG"
    qual = "#####"
    segment = 'read1'
    if_tag = "is_short"  # Append postfix only to short reads
```

## See Also

- [Tag extraction reference]({{< relref "docs/reference/tag-steps/_index.md" >}}) for all tag-generating steps
- [Step concept]({{< relref "docs/concepts/step.md" >}}) for tag lifecycle validation
- [Source concept]({{< relref "docs/concepts/source.md" >}}) for using tags as data sources