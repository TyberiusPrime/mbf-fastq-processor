# Step

A step is one coherent manipulation of the FASTQ stream and its associated data.

## Overview

Steps are the building blocks of a processing pipeline. Each step is declared as a `[[step]]` entry in the TOML configuration file, and the complete pipeline executes steps sequentially from top to bottom.

Every step operates on complete fragments (molecules), ensuring that paired segments remain synchronized. If a filtering step removes a fragment based on criteria from `read1`, the corresponding `read2`, `index1`, and any other segments are automatically removed alongside it.

## Step Categories

mbf-fastq-processor organizes steps into five functional categories:

### Modification Steps

Modify the content or structure of fragments:
- Trimming sequences (e.g., `CutStart`, `CutEnd`, `TrimAtTag`)
- Transforming sequences (e.g., `ReverseComplement`, `UppercaseSequence`, `LowercaseSequence`)
- Restructuring fragments (e.g., `Swap`, `MergeReads`)
- Manipulating read names (e.g., `Rename`, `Prefix`, `Postfix`)
- Limiting throughput (e.g., `Head`, `Skip`)

Modification steps change the data but preserve fragment identity and pairing.

### Filter Steps

Remove fragments based on criteria:
- Content-based filtering (e.g., `FilterByTag`, `FilterByNumericTag`)
- Statistical sampling (e.g., `FilterSample`, `FilterReservoirSample`)
- Structural validation (e.g., `FilterEmpty`)

Filter steps reduce the fragment count and may dramatically alter downstream statistics.

### Tag Steps

Extract, calculate, or transform metadata into labeled tags:
- **Extract**: Locate patterns or regions in sequences (e.g., `ExtractIUPAC`, `ExtractRegex`)
- **Calc**: Compute numeric metrics (e.g., `CalcMeanQuality`, `CalcGCContent`)
- **Convert**: Transform existing tags (e.g., `EvalExpression`, `ConvertTagToNumeric`)
- **Tag**: Assign boolean flags (e.g., `TagIfTagPresent`, `TagByLength`)

Tag steps create labeled metadata that downstream steps can consume, modify, or export.

### Report Steps

Gather statistics and produce outputs without modifying the stream:
- Quality metrics (e.g., `Report`)
- Sample inspection (e.g., `Inspect`)
- Progress tracking (e.g., `Progress`)
- Data export (e.g., `QuantifyTag`, `StoreTagsInTable`)

Report steps are non-destructive and can be placed at multiple points in the pipeline to observe intermediate states.

### Validation Steps

Assert correctness and detect data quality issues:
- Format validation (e.g., `ValidateSeq`, `ValidateQuality`, `ValidateName`)
- Pairing verification (e.g., `SpotCheckReadPairing`)
- Length consistency (e.g., `ValidateAllReadsSameLength`)

Validation steps halt execution with detailed error messages when assertions fail, making them invaluable for debugging and quality control.

## Step Ordering and Execution

Steps execute in the exact order they appear in the TOML file. Order matters:

```toml
[[step]]
name = "Report"      # Measure quality before filtering

[[step]]
name = "FilterByNumericTag"
in_label = "mean_q"
minimum = 20

[[step]]
name = "Report"      # Measure quality after filtering
```

The same step type can appear multiple times with different parameters, allowing you to incrementally transform data or collect statistics at different pipeline stages.

## Segment Targeting

Many steps accept a `segment` parameter to restrict their operation to a specific input stream:

```toml
[[step]]
name = "CutStart"
segment = "read1"    # Only trim read1
length = 10

[[step]]
name = "FilterByLength"
segment = "All"      # Evaluate all segments
minimum = 50
```

Even when targeting a specific segment, the step operates on the entire fragment and will filter/modify all associated segments together.

## Tag Lifecycle

Steps that produce tags (via `out_label`) must be paired with steps that consume those tags (via `in_label` or `in_labels`). The processor validates this at startup and will error if:
- A tag is created but never used
- A tag is referenced but never defined

This validation catches typos early and ensures pipeline correctness.

## See Also

- [Reference documentation]({{< relref "docs/reference/_index.md" >}}) for detailed parameter specifications
- [Tag concept]({{< relref "docs/concepts/tag.md" >}}) for tag workflow patterns
- [Segments concept]({{< relref "docs/concepts/segments.md" >}}) for multi-segment processing
