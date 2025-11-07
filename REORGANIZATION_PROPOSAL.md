# Test Case Reorganization Proposal

## Overview

This reorganization consolidates 455 integration tests into a more logical structure based on **what feature is being tested** rather than the original somewhat arbitrary categorization.

**Key Changes:**
- **312 tests will be moved** to new locations
- **143 tests will stay** in their current locations
- Old structure had 15 top-level categories, new structure has **29 more focused categories**

## Reorganization Summary

### Current Problems

1. **`integration_tests/`** is a catch-all (124 tests) with no clear organization
2. **`input_validation/`** mixes different error types (169 tests)
3. **Inconsistent naming** (e.g., `hamming_correct/` vs `head_early_termination/`)
4. **Feature tests scattered** across multiple directories

### New Organization Principles

1. **Feature-first**: Group by the primary feature being tested
2. **Error separation**: All error/validation tests in `error_handling/`
3. **Edge cases separated**: Unusual formats, challenging data in `edge_cases/`
4. **Clear naming**: Concise, consistent category names

## New Category Structure

### Core Feature Tests (265 tests)

| Category | Count | Description |
|----------|-------|-------------|
| `extraction/` | 81 | Tag/region extraction, UMI extraction, regex extraction |
| `filter/` | 22 | Read filtering by quality, length, complexity, etc. |
| `demultiplex/` | 19 | Barcode demultiplexing, tag-based splitting |
| `output/` | 12 | Output formatting, chunking, interleaving |
| `head/` | 11 | Head operation and early termination |
| `reports/` | 10 | Report generation and statistics |
| `fileformats/` | 9 | Format conversions (BAM/FASTA/FASTQ) |
| `edits/` | 9 | Sequence editing (uppercase/lowercase/merge) |
| `compression/` | 10 | Compression handling (gzip/zstd) |
| `io/` | 9 | stdin/stdout/interleaved I/O |
| `trim/` | 8 | Trimming and cutting operations |
| `dedup/` | 8 | Deduplication |
| `calc/` | 7 | Calculations (expected error, k-mers, quantification) |
| `convert/` | 7 | Data conversions (regions to length, rates) |
| `transform/` | 7 | Sequence transformations (reverse complement, rename, etc.) |
| `hamming/` | 4 | Hamming distance corrections |
| `inspect/` | 4 | File inspection |
| `eval/` | 3 | Expression evaluation |
| `basic/` | 3 | Basic functionality (noop, overwrite, etc.) |
| `quality/` | 3 | Quality score handling |
| `validation/` | 13 | Runtime validation (hash checks, pairing) |
| `sampling/` | 2 | Subsampling and skipping |
| `swap/` | 1 | Segment swapping |
| `correctness/` | 1 | Ordering/correctness tests |
| `performance/` | 1 | Memory/performance tests |
| `compatibility/` | 4 | Legacy formats, fastp compatibility |

### Error & Edge Cases (174 tests)

| Category | Count | Description |
|----------|-------|-------------|
| `error_handling/` | 154 | All validation errors, organized by subsystem |
| `edge_cases/` | 20 | Challenging formats, unusual inputs |

#### Error Handling Subcategories

The `error_handling/` category is organized by what subsystem is being validated:

- `error_handling/input_files/` - File path, permission, format errors
- `error_handling/malformed_fastq/` - Broken FASTQ files (truncated, invalid quality, etc.)
- `error_handling/output_config/` - Output configuration errors
- `error_handling/extraction/` - Tag/extraction validation
- `error_handling/demultiplex/` - Barcode validation
- `error_handling/filter/` - Filter parameter validation
- `error_handling/bam/` - BAM-specific errors
- `error_handling/compression/` - Compression parameter errors
- `error_handling/cli/` - Command-line interface errors
- `error_handling/misc/` - Other validation errors

### Integration Tests (13 tests)

Tests that don't fit into specific feature categories remain in `integration/` for general end-to-end testing.

## Examples of Major Changes

### From `integration_tests/` → Feature Categories

**Before:** Everything in `integration_tests/`
```
integration_tests/filter_empty
integration_tests/filter_min_len
integration_tests/dedup/basic
integration_tests/trim_poly_tail_detail
```

**After:** Organized by feature
```
filter/filter_empty
filter/filter_min_len
dedup/dedup/basic
trim/trim_poly_tail_detail
```

### From `input_validation/` → Organized Errors

**Before:** All mixed in `input_validation/`
```
input_validation/missing_input_file
input_validation/bam_output_uncompressed_hash
input_validation/extract_tag_duplicate_name_panics
input_validation/broken_newline
```

**After:** Organized by subsystem
```
error_handling/input_files/missing_input_file
error_handling/bam/bam_output_uncompressed_hash
error_handling/extraction/extract_tag_duplicate_name_panics
error_handling/malformed_fastq/broken_newline
```

### Category Renames

- `hamming_correct/` → `hamming/` (more concise)
- `head_early_termination/` → `head/` (more concise)
- `memory/` → `performance/` (more descriptive)

## Benefits

1. **Easier navigation**: Find tests by feature, not arbitrary category
2. **Better organization**: Clear separation of success vs error tests
3. **Improved maintenance**: Related tests are grouped together
4. **Clearer intent**: Category names indicate what's being tested
5. **Scalability**: Easy to add new tests in appropriate categories

## Files Generated

1. **`reorganize_tests.sh`** (2,285 lines)
   - Executable bash script with all rename commands
   - Includes safety confirmation prompt
   - Creates parent directories as needed
   - Colored output for visibility

2. **`test_reorganization_plan.txt`** (1,728 lines)
   - Detailed list of every rename
   - Includes reasoning for each change
   - Grouped by old and new categories for easy review

## How to Review

1. **Quick overview**: Read this document (you're doing it!)

2. **Detailed review**: Open `test_reorganization_plan.txt`
   ```bash
   less test_reorganization_plan.txt
   ```

3. **Check specific categories**:
   ```bash
   grep "input_validation/broken" test_reorganization_plan.txt
   ```

4. **Review the script**:
   ```bash
   less reorganize_tests.sh
   ```

## How to Execute

**⚠️ IMPORTANT: Review carefully before running!**

```bash
# 1. Review the plan
less test_reorganization_plan.txt

# 2. Run the reorganization script
./reorganize_tests.sh

# 3. Update test infrastructure
dev/update_tests.py

# 4. Run tests to verify
cargo test

# 5. Commit the changes
jj commit -m "Reorganize integration tests by feature"
```

## Statistics

### Migration Summary

| Old Category | Tests | Primary Destination(s) |
|--------------|-------|----------------------|
| `integration_tests/` | 124 | Split across 21 categories |
| `input_validation/` | 169 | → `error_handling/` (152), `edge_cases/` (17) |
| `extraction/` | 76 | Mostly stays in `extraction/` |
| `demultiplex/` | 17 | Stays in `demultiplex/` |
| `head_early_termination/` | 11 | → `head/` |
| `reports/` | 10 | Stays in `reports/` |
| `edits/` | 9 | Stays in `edits/` |
| `calc/` | 5 | + 2 from integration → `calc/` (7 total) |
| `convert/` | 7 | Stays in `convert/` |
| `fileformats/` | 7 | + 2 from integration → `fileformats/` (9 total) |
| `hamming_correct/` | 4 | → `hamming/` |
| `validation/` | 11 | Mostly stays in `validation/` |
| `output/` | 2 | + 10 from integration → `output/` (12 total) |
| `outside_error_conditions/` | 2 | → `error_handling/` |
| `memory/` | 1 | → `performance/` |

### Complexity Reduction

- **Before**: 1 category with 169 tests (`input_validation/`)
- **After**: Largest category has 81 tests (`extraction/`)
- **Average category size**: 15.7 tests (was 30.3)

## Questions?

If you find any categorization that seems wrong or have suggestions for improvement, they can be adjusted before execution. The Python script that generates these files can be easily modified.
