status: open
# PE to SE with Overlap Analysis Comparison

- **Objective**: Compare our overlap detection with fastp's implementation
- **Technical Details**:
  - fastp uses simple offset checking for overlap detection with parameters:
    - `overlap_len_require` (default 30)
    - `overlap_diff_limit` (default 5)
    - `overlap_diff_percent_limit` (default 20%)
  - Our approach: Modified Smith-Waterman algorithm from rust-bio
- **Expected Outcome**: Show we're both more accurate and faster
- **Requirements**: Need test datasets for evaluation

Also consider setting the incongruent bases to N,
see https://github.com/OpenGene/fastp/issues/346
