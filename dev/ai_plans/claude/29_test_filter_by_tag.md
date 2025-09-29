# Plan: Add Test Case for FilterByTag Location Tag Validation

## Goal
Add test case that validates `FilterByTag` only accepts location tags, rejecting numeric and boolean tags with appropriate error messages.

## Background Analysis

### Current FilterByTag Implementation
- **Location**: `src/transformations/filters/by_tag.rs:28`
- **Validation**: Uses `validate_tag_set_and_type()` requiring `TagValueType::Location`
- **Error format**: `"Step expects Location tag named '{label}', but earlier step declares {actual_type} tag"`

### Existing Template Analysis
- **Template**: `test_cases/input_validation/filter_no_such_tag/`
- **Structure**: 
  - `input.toml` - TOML config that triggers validation error
  - `expected_panic.txt` - Expected error message
- **Current test**: Validates missing tag error: `"Step expects Location tag named 'test', but no earlier step declares this tag"`

### Tag Types in System
- **Location tags**: ExtractIUPAC, ExtractRegion, ExtractAnchor, ExtractRegex, etc.
- **Numeric tags**: ExtractGCContent, ExtractLength, ExtractNCount, ExtractMeanQuality, etc.  
- **Boolean tags**: ExtractTagDuplicates, ExtractTagOtherFileByName, ExtractTagOtherFileBySequence

## Implementation Plan

### 1. Create Test Case: `filter_by_tag_numeric_rejection`

**Location**: `test_cases/input_validation/filter_by_tag_numeric_rejection/`

**Files to create**:
- `input.toml` - Pipeline attempting to filter by numeric tag
- `expected_panic.txt` - Expected validation error message

**TOML Configuration**:
```toml
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractGCContent'
    label = 'gc_content'

[[step]]
    action = 'FilterByTag'
    keep_or_remove = 'Keep'
    label = 'gc_content'

[output]
    prefix = 'output'
```

**Expected error**: `"Step expects Location tag named 'gc_content', but earlier step declares Numeric tag". Use FilterByNumericTag for numeric tags."`

### 2. Create Test Case: `filter_by_tag_bool_rejection`

**Location**: `test_cases/input_validation/filter_by_tag_bool_rejection/`

**Files to create**:
- `input.toml` - Pipeline attempting to filter by boolean tag  
- `expected_panic.txt` - Expected validation error message

**TOML Configuration**:
```toml
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractTagDuplicates'
    label = 'is_duplicate'

[[step]]
    action = 'FilterByTag'
    keep_or_remove = 'Keep'
    label = 'is_duplicate'

[output]
    prefix = 'output'
```

**Expected error**: `"Step expects Location tag named 'is_duplicate', but earlier step declares Bool tag". Use FilterByBoolTag for boolean tags."`


## Test Strategy

### Validation Tests (Panic Expected)
1. **Numeric Tag Rejection**: FilterByTag + ExtractGCContent
2. **Boolean Tag Rejection**: FilterByTag + ExtractTagDuplicates  
3. **Missing Tag**: Already covered by existing `filter_no_such_tag`

### Error Message Verification
- Verify exact error format matches: `"Step expects {expected_type} tag named '{label}', but earlier step declares {actual_type} tag"`
- Ensure error occurs during validation phase, not runtime

## Implementation Steps

1. **Create directory structure** for three new test cases
2. **Write TOML configurations** based on the templates above
3. **Create expected_panic.txt** files with precise error messages
4. **Run test generation**: `dev/update_tests.py` 
5. **Execute tests**: `cargo test` to verify all cases work correctly
6. **Validate error messages** match expected format exactly

## Testing Verification

### Commands to run:
```bash
# Generate test cases
dev/update_tests.py

# Run specific new tests  
cargo test filter_by_tag_numeric_rejection
cargo test filter_by_tag_bool_rejection
cargo test filter_by_tag_location_acceptance

# Run all tests to ensure no regressions
cargo test
```

### Expected outcomes:
- ✅ `filter_by_tag_numeric_rejection` - Should panic with numeric tag error
- ✅ `filter_by_tag_bool_rejection` - Should panic with boolean tag error  
- ✅ `filter_by_tag_location_acceptance` - Should complete successfully with filtered output

## Success Criteria

1. **Type validation enforced**: FilterByTag rejects non-location tags during validation
2. **Clear error messages**: Users get informative error messages about tag type mismatches
3. **No regressions**: All existing tests continue to pass
4. **Test coverage**: Both positive and negative cases covered for FilterByTag tag type validation

This plan ensures comprehensive testing of FilterByTag's tag type validation requirements while following the existing test case patterns and directory structure.
