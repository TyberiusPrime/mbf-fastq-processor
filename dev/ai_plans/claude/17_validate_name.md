# outcome: not attempted
# ValidateName Step Implementation Plan

## Overview
Implement a `ValidateName` validation step that ensures read names across all present reads (read1/read2/index1/index2) are either identical or share a common prefix up to the first occurrence of `readname_end_chars`.

## Implementation Steps

### 1. Create ValidateName struct (`src/transformations/validation/name.rs`)
- Define `ValidateName` struct with optional `readname_end_chars` field
- Use `BString` type for the separator characters (following existing pattern from `default_name_separator()`)
- Default value should be `"_"` (underscore) to match existing Illumina conventions
- Implement serde deserialization with `#[serde(deny_unknown_fields)]`

### 2. Implement Step trait for ValidateName
- **validate()**: Check that input definition has required reads available
- **apply()**: Core validation logic
  - For each read position (0..block_size), extract names from all present reads
  - Compare names using either exact match or prefix comparison logic
  - Use assertion with descriptive error message on validation failure (following `ValidateSeq` pattern)
- **needs_serial()**: Return `false` (can run in parallel)
- **transmits_premature_termination()**: Return `true` (default behavior)

### 3. Name comparison logic
- If `readname_end_chars` is None: require exact name matching across all reads
- If `readname_end_chars` is Some(chars): 
  - Find first occurrence of any character from `chars` in read1 name
  - Extract prefix up to that position
  - Verify all other read names have the same prefix
  - Handle edge cases: no separator found, empty names, etc.

### 4. Add to transformation enum (`src/transformations.rs`)
- Add `ValidateName(validation::ValidateName)` variant to the `Transformation` enum
- Update the validation module export in `src/transformations/validation.rs`

### 5. Create test cases
- **Basic identical names**: All reads have exactly the same name
- **Prefix matching with underscore**: Names like "READ001_1", "READ001_2", "READ001_I1", "READ001_I2"  
- **Custom separator characters**: Test with different `readname_end_chars` values
- **Validation failures**: 
  - Names with different prefixes
  - Names with same prefix but different separators
  - Mixed exact/prefix scenarios
- **Edge cases**: Empty names, no separator found, single read type only
- **Multi-read scenarios**: Test with all combinations of read1/read2/index1/index2 present

### 6. Test case structure
Each test case should include:
- `input.toml` with ValidateName step configuration
- Input FastQ files with appropriate read names
- Expected behavior: success (pass-through) or failure (assertion error)
- Place in `test_cases/validation/validate_name/` directory

### 7. Update main transformation module
- Add import for `ValidateName` in `src/transformations/validation.rs` 
- Ensure the new validation step is properly integrated into the transformation pipeline

## Files to create/modify:

### New files:
1. `src/transformations/validation/name.rs` - Main implementation
2. `test_cases/validation/validate_name/*/input.toml` - Test configurations
3. `test_cases/validation/validate_name/*/input_*.fq` - Test FastQ files

### Modified files:
1. `src/transformations.rs` - Add ValidateName to Transformation enum
2. `src/transformations/validation.rs` - Export ValidateName module and struct

## Expected behavior:
- **Success**: Reads pass through unchanged when names validate correctly
- **Failure**: Process terminates with assertion error showing problematic read names
- **Performance**: Can run in parallel, no special serialization needs
- **Integration**: Works with existing pipeline, no demultiplexing or tagging interactions
