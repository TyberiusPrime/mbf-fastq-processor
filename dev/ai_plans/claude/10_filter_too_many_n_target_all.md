# outcome: success
# AI Plan: Extend FilterTooManyN to Support target=all

## Current Implementation Analysis

The current `FilterTooManyN` implementation in `src/transformations/filters.rs:274-302` uses `apply_filter_plus_all` which applies an OR logic - if any single segment passes the filter, the entire fragment is kept. For `target=all`, we need AND logic - count N's across all segments and filter based on the sum.

## Required Changes

### 1. Modify FilterTooManyN Implementation (src/transformations/filters.rs:289-301)

Current implementation:
```rust
fn apply(
    &mut self,
    mut block: crate::io::FastQBlocksCombined,
    _block_no: usize,
    _demultiplex_info: &Demultiplexed,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_filter_plus_all(self.target, &mut block, |read| {
        let seq = read.seq();
        let sum: usize = seq.iter().map(|x| usize::from(*x == b'N')).sum();
        sum <= self.n
    });
    (block, true)
}
```

**Change to:**
```rust
fn apply(
    &mut self,
    mut block: crate::io::FastQBlocksCombined,
    _block_no: usize,
    _demultiplex_info: &Demultiplexed,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_filter_plus_all_ext(
        self.target,
        &mut block,
        |read| {
            let seq = read.seq();
            let sum: usize = seq.iter().map(|x| usize::from(*x == b'N')).sum();
            sum <= self.n
        },
        |read1, opt_read2, opt_i1, opt_i2| {
            let mut total_ns = 0;
            
            // Count N's in read1
            total_ns += read1.seq().iter().map(|x| usize::from(*x == b'N')).sum::<usize>();
            
            // Count N's in read2 if present
            if let Some(read2) = opt_read2 {
                total_ns += read2.seq().iter().map(|x| usize::from(*x == b'N')).sum::<usize>();
            }
            
            // Count N's in index1 if present
            if let Some(i1) = opt_i1 {
                total_ns += i1.seq().iter().map(|x| usize::from(*x == b'N')).sum::<usize>();
            }
            
            // Count N's in index2 if present
            if let Some(i2) = opt_i2 {
                total_ns += i2.seq().iter().map(|x| usize::from(*x == b'N')).sum::<usize>();
            }
            
            total_ns <= self.n
        },
    );
    (block, true)
}
```

### 2. Create Test Cases

Based on the pattern from `filter_empty_all` and `filter_empty_segments`, create two new test cases:

#### Test Case 1: `filter_too_many_n_all`
- **Purpose**: Test filtering when `target=all` sums N's across all segments
- **Input files**: 4 files (read1, read2, index1, index2) with varying N content
- **Configuration**: `FilterTooManyN` with `target = "all"` and threshold that filters some but not all reads
- **Expected**: Filter based on sum of N's across all segments

#### Test Case 2: `filter_too_many_n_segments_vs_all`  
- **Purpose**: Test difference between single segment filtering and all-segment filtering
- **Input files**: Same 4 files as test case 1
- **Configuration**: Two steps - one filtering `target = "read1"`, another filtering `target = "all"`
- **Expected**: Different results showing single-segment vs. sum-of-all-segments logic

### 3. Test File Creation Strategy

**Base input files on `filter_empty_all` but replace some bases with 'N':**

- Use existing pattern from `filter_empty_all/input_*.fq` files
- Replace specific bases with 'N' to create controlled N-content scenarios
- Ensure some reads pass individual segment thresholds but fail when all segments are summed
- Ensure some reads fail individual segment thresholds but pass when segments compensate

### 4. Expected Test Scenarios

**Scenario A**: Read with few N's in each segment individually, but sum exceeds threshold
- read1: "ATNCGN" (2 N's)
- read2: "GCNATN" (2 N's)  
- index1: "NNCG" (2 N's)
- index2: "ATNN" (2 N's)
- Total: 8 N's - should be filtered if threshold < 8, but individual segments might pass if threshold ≥ 2

**Scenario B**: Read with many N's in one segment, few in others
- read1: "NNNNNN" (6 N's)
- read2: "ATCGATCG" (0 N's)
- index1: "ATCG" (0 N's) 
- index2: "GCTA" (0 N's)
- Total: 6 N's - individual read1 filtering would remove, but sum might be acceptable

## Implementation Steps

1. **Modify the FilterTooManyN apply() method** to use `apply_filter_plus_all_ext`
2. **Create test input files** by modifying existing `filter_empty_all` files
3. **Write test configurations** following the existing pattern
4. **Generate expected outputs** by running the modified implementation
5. **Add test cases to integration test suite** using `dev/update_tests.py`

## Files to Create/Modify

### Modify:
- `src/transformations/filters.rs` (lines 289-301)

### Create:
- `test_cases/integration_tests/filter_too_many_n_all/`
  - `input.toml`
  - `input_read1.fq`
  - `input_read2.fq`
  - `input_index1.fq`
  - `input_index2.fq`

- `test_cases/integration_tests/filter_too_many_n_segments_vs_all/`
  - `input.toml`  
  - `input_read1.fq`
  - `input_read2.fq`
  - `input_index1.fq`
  - `input_index2.fq`

The implementation follows the same pattern used by `FilterEmpty` which already successfully uses `apply_filter_plus_all_ext` for `target=all` logic.
