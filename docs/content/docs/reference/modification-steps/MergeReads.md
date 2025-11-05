# MergeReads

```toml
[[step]]
    action = "MergeReads"
    algorithm = "fastp"                   # Algorithm: "fastp", "simple_bayes", or "rdp_mle" (required)
    min_overlap = 30                      # Minimum overlap length required
    max_mismatch_rate = 0.2               # (optional) Maximum allowed mismatch rate (0.0-1.0) (suggested: 0.2)
    max_mismatch_count = 5                # (optional) Maximum allowed absolute mismatches (suggested: 5)
                                          # At least one of max_mismatch_rate or max_mismatch_count required
    allow_gap = false                     # Allow single gap/indel in alignment (suggested: false)
    no_overlap_strategy = "as_is"         # What to do when no overlap found: "as_is" or "concatenate" (suggested: "as_is")
    label = "merged"                      # (optional) Tag label for merge status (suggested: "merged")
    reverse_complement_segment2 = true    # Whether to reverse complement segment2 (suggested: true)
    segment1 = "read1"                    # First segment (suggested: "read1")
    segment2 = "read2"                    # Second segment (suggested: "read2")
    concatenate_spacer = "NNNN"           # (optional) Required if no_overlap_strategy = "concatenate". Spacer sequence to insert between reads
    spacer_quality_char = 33              # (optional) Quality score for spacer bases (suggested: 33 = Phred quality 0)
```

Merges paired-end reads from two segments by detecting their overlap and resolving mismatches. Supports multiple algorithms: fastp (quality-score based), simple_bayes (Bayesian probability from pandaseq), and rdp_mle (Maximum Likelihood Estimation from pandaseq).

## How it works

1. Optionally takes the reverse complement of segment2 (controlled by `reverse_complement_segment2`)
2. Searches for overlap between segment1 and the processed segment2
3. If overlap is found:
   - Merges the reads using quality scores to resolve mismatches
   - Places merged sequence in segment1
   - Empties segment2
4. If no overlap is found:
   - **as_is**: Leaves both segments unchanged
   - **concatenate**: Joins segment1 + spacer + processed segment2 into segment1, empties segment2
5. If `label` is specified, creates a boolean tag indicating merge status (true=merged, false=not merged)

## Parameters

- **algorithm** (required): Algorithm to use for overlap scoring and mismatch resolution. Options:
  - `"fastp"`: Quality-score based mismatch resolution using hamming distance for overlap detection. Chooses higher quality base for mismatches.
  - `"simple_bayes"`: Bayesian probability algorithm from pandaseq. Uses default error probability q=0.36.
  - `"rdp_mle"`: Maximum Likelihood Estimation algorithm from pandaseq. Optimized for MiSeq data, adjusts for observed error patterns.

- **min_overlap** (required): Minimum number of overlapping bases required for merging. Suggested: 30.

- **max_mismatch_rate** (optional): Maximum allowed mismatch rate in the overlap region (0.0 = perfect match, 1.0 = allow all mismatches). Suggested: 0.2 (20%). At least one of `max_mismatch_rate` or `max_mismatch_count` must be specified. Note: This parameter applies only to the `fastp` algorithm.

- **max_mismatch_count** (optional): Maximum allowed absolute number of mismatches in the overlap region. Suggested: 5. At least one of `max_mismatch_rate` or `max_mismatch_count` must be specified. If both are specified, both conditions must be met (AND logic). Note: This parameter applies only to the `fastp` algorithm.

- **allow_gap** (required): Enable detection of single insertion/deletion in the overlap region. Suggested: false.

- **no_overlap_strategy** (required): Strategy when no overlap is detected:
  - `"as_is"` (suggested): Leave reads unchanged
  - `"concatenate"`: Join reads with spacer sequence

- **label** (optional): Tag label to store merge status as a boolean tag. If specified, creates a tag where `true` indicates the reads were merged and `false` indicates they were not merged. Suggested: "merged". This tag can be used later for demultiplexing or filtering.

- **reverse_complement_segment2** (required): Whether to reverse complement segment2 before processing. Suggested: true (for standard paired-end reads).

- **segment1**, **segment2** (required): Names of segments to merge. Suggested: "read1", "read2".

- **concatenate_spacer** (optional, required if no_overlap_strategy = "concatenate"): Sequence to insert between reads when concatenating. Common values: `""` (empty), `"NNNN"`.

- **spacer_quality_char** (optional): ASCII quality score character for spacer bases. Suggested: 33 (Phred quality 0).

## Example: Basic merging with fastp

```toml
[[step]]
    action = "MergeReads"
    algorithm = "fastp"
    min_overlap = 30
    max_mismatch_rate = 0.2
    allow_gap = false
    no_overlap_strategy = "as_is"
    reverse_complement_segment2 = true
    segment1 = "read1"
    segment2 = "read2"
```

Merges read1 and read2 using fastp algorithm when they have at least 30bp overlap with ≤20% mismatches. Reads without sufficient overlap remain unchanged.

## Example: Bayesian merging

```toml
[[step]]
    action = "MergeReads"
    algorithm = "simple_bayes"
    min_overlap = 30
    max_mismatch_count = 5
    allow_gap = false
    no_overlap_strategy = "as_is"
    reverse_complement_segment2 = true
    segment1 = "read1"
    segment2 = "read2"
```

Uses pandaseq's Bayesian probability algorithm for more sophisticated overlap scoring based on quality scores.

## Example: Concatenate non-overlapping reads

```toml
[[step]]
    action = "MergeReads"
    algorithm = "fastp"
    min_overlap = 30
    max_mismatch_rate = 0.2
    allow_gap = false
    no_overlap_strategy = "concatenate"
    concatenate_spacer = "NNNN"
    reverse_complement_segment2 = true
    segment1 = "read1"
    segment2 = "read2"
```

Merges overlapping reads, but concatenates non-overlapping reads with "NNNN" spacer into segment1.

## Example: Strict merging with RDP MLE

```toml
[[step]]
    action = "MergeReads"
    algorithm = "rdp_mle"
    min_overlap = 50
    max_mismatch_count = 2
    allow_gap = false
    no_overlap_strategy = "as_is"
    reverse_complement_segment2 = true
    segment1 = "read1"
    segment2 = "read2"
```

Uses pandaseq's RDP MLE algorithm optimized for MiSeq data. Requires 50bp overlap for merging. More conservative, fewer false merges.

## Notes

- Set `reverse_complement_segment2 = true` for standard paired-end reads where read2 is the reverse complement
- In overlapping regions, bases with higher quality scores are chosen
- After merging/concatenation, segment2 is always emptied
- The `allow_gap` parameter enables detection of single insertions/deletions but may be slower
- Use `concatenate_spacer = ""` for direct concatenation without spacer
