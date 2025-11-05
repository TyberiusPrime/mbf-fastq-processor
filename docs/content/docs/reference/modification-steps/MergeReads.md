# MergeReads

```toml
[[step]]
    action = "MergeReads"
    min_overlap = 30                      # Minimum overlap length required
    max_mismatch_rate = 0.2               # Maximum allowed mismatch rate (0.0-1.0)
    allow_gap = false                     # Allow single gap/indel in alignment (suggested: false)
    no_overlap_strategy = "keep"          # What to do when no overlap found: "keep" or "concatenate" (suggested: "keep")
    reverse_complement_segment2 = true    # Whether to reverse complement segment2 (suggested: true)
    segment1 = "read1"                    # First segment (suggested: "read1")
    segment2 = "read2"                    # Second segment (suggested: "read2")
    concatenate_spacer = "NNNN"           # (optional) Required if no_overlap_strategy = "concatenate". Spacer sequence to insert between reads
    spacer_quality_char = 33              # (optional) Quality score for spacer bases (suggested: 33 = Phred quality 0)
```

Merges paired-end reads from two segments by detecting their overlap and resolving mismatches using quality scores. Based on the fastp overlap analysis algorithm.

## How it works

1. Optionally takes the reverse complement of segment2 (controlled by `reverse_complement_segment2`)
2. Searches for overlap between segment1 and the processed segment2
3. If overlap is found:
   - Merges the reads using quality scores to resolve mismatches
   - Places merged sequence in segment1
   - Empties segment2
4. If no overlap is found:
   - **keep**: Leaves both segments unchanged
   - **concatenate**: Joins segment1 + spacer + processed segment2 into segment1, empties segment2

## Parameters

- **min_overlap** (required): Minimum number of overlapping bases required for merging. Suggested: 30.

- **max_mismatch_rate** (required): Maximum allowed mismatch rate in the overlap region (0.0 = perfect match, 1.0 = allow all mismatches). Suggested: 0.2 (20%).

- **allow_gap** (required): Enable detection of single insertion/deletion in the overlap region. Suggested: false.

- **no_overlap_strategy** (required): Strategy when no overlap is detected:
  - `"keep"` (suggested): Leave reads unchanged
  - `"concatenate"`: Join reads with spacer sequence

- **reverse_complement_segment2** (required): Whether to reverse complement segment2 before processing. Suggested: true (for standard paired-end reads).

- **segment1**, **segment2** (required): Names of segments to merge. Suggested: "read1", "read2".

- **concatenate_spacer** (optional, required if no_overlap_strategy = "concatenate"): Sequence to insert between reads when concatenating. Common values: `""` (empty), `"NNNN"`.

- **spacer_quality_char** (optional): ASCII quality score character for spacer bases. Suggested: 33 (Phred quality 0).

## Example: Basic merging

```toml
[[step]]
    action = "MergeReads"
    min_overlap = 30
    max_mismatch_rate = 0.2
    allow_gap = false
    no_overlap_strategy = "keep"
    reverse_complement_segment2 = true
    segment1 = "read1"
    segment2 = "read2"
```

Merges read1 and read2 when they have at least 30bp overlap with ≤20% mismatches. Reads without sufficient overlap remain unchanged.

## Example: Concatenate non-overlapping reads

```toml
[[step]]
    action = "MergeReads"
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

## Example: Strict merging

```toml
[[step]]
    action = "MergeReads"
    min_overlap = 50
    max_mismatch_rate = 0.05
    allow_gap = false
    no_overlap_strategy = "keep"
    reverse_complement_segment2 = true
    segment1 = "read1"
    segment2 = "read2"
```

Requires 50bp overlap with ≤5% mismatches for merging. More conservative, fewer false merges.

## Notes

- Set `reverse_complement_segment2 = true` for standard paired-end reads where read2 is the reverse complement
- In overlapping regions, bases with higher quality scores are chosen
- After merging/concatenation, segment2 is always emptied
- The `allow_gap` parameter enables detection of single insertions/deletions but may be slower
- Use `concatenate_spacer = ""` for direct concatenation without spacer
