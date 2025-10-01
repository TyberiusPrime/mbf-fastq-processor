# outcome: success
# This worked.

# AI Plan 23: Swap Step Optional Segments Enhancement

## Goal
Enhance the `Swap` step to allow omitting the `segment_a` and `segment_b` parameters when exactly two input segments are defined. Add validation to ensure at least two segments exist and prevent partial specification of segments.

## Background
Currently, the `Swap` step requires explicit specification of both `segment_a` and `segment_b` parameters. When working with simple two-segment inputs (e.g., read1 and read2), users must explicitly specify which segments to swap, even though there's only one logical choice.

## Current Implementation Analysis
- **Location**: `/project/src/transformations/edits/swap.rs`
- **Structure**: Uses `Segment` wrapper struct with required `segment_a` and `segment_b` fields
- **Validation**: Currently validates segments exist and are different
- **Test Cases**: Existing tests in `/project/test_cases/` cover error cases for missing segments

## Proposed Changes

### 1. Configuration Structure Enhancement
**File**: `/project/src/transformations/edits/swap.rs`

Modify the `Swap` struct to make segments optional:
```rust
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Swap {
    #[serde(default)]
    segment_a: Option<Segment>,
    #[serde(default)]
    #[serde(skip)]
    segment_a_index: Option<SegmentIndex>,

    #[serde(default)]  
    segment_b: Option<Segment>,
    #[serde(default)]
    #[serde(skip)]
    segment_b_index: Option<SegmentIndex>,
}
```

### 2. Enhanced Validation Logic
**File**: `/project/src/transformations/edits/swap.rs:25`

Update `validate_segments` method:
```rust
fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
    let segment_count = input_def.segment_count();
    
    // Case 1: Both segments specified explicitly
    if let (Some(seg_a), Some(seg_b)) = (&self.segment_a, &self.segment_b) {
        if seg_a == seg_b {
            bail!("Swap was supplied the same segment for segment_a and segment_b");
        }
        self.segment_a_index = Some(seg_a.validate(input_def)?);
        self.segment_b_index = Some(seg_b.validate(input_def)?);
        return Ok(());
    }
    
    // Case 2: Auto-detect for exactly two segments
    if self.segment_a.is_none() && self.segment_b.is_none() {
        if segment_count != 2 {
            bail!("Swap requires exactly 2 input segments when segment_a and segment_b are omitted, but {} segments were provided", segment_count);
        }
        
        let segment_order = input_def.get_segment_order();
        let mut seg_a = Segment(segment_order[0].clone());
        let mut seg_b = Segment(segment_order[1].clone());
        
        self.segment_a_index = Some(seg_a.validate(input_def)?);
        self.segment_b_index = Some(seg_b.validate(input_def)?);
        self.segment_a = Some(seg_a);
        self.segment_b = Some(seg_b);
        return Ok(());
    }
    
    // Case 3: Partial specification error
    if self.segment_a.is_some() || self.segment_b.is_some() {
        bail!("Swap requires both segment_a and segment_b to be specified, or both to be omitted for auto-detection with exactly 2 segments");
    }
    
    Ok(())
}
```

### 3. Updated Apply Method
**File**: `/project/src/transformations/edits/swap.rs:35`

No significant changes needed to the `apply` method as it already uses the validated indices.

### 4. Test Cases

#### 4.1 Auto-detection Success Test
**File**: `/project/test_cases/integration_tests/swap_auto_detect_two_segments/input.toml`
```toml
[input]
    read1 = ['input_read1.fq']
    read2 = ['input_read2.fq']

[[step]]
    action='Head'
    n = 5

[[step]]
    action = 'Swap'
    # No segment_a/segment_b specified - should auto-detect

[output]
    prefix = 'output'
```

#### 4.2 Auto-detection Error Test (Too Many Segments)
**File**: `/project/test_cases/input_validation/swap_auto_detect_too_many_segments/input.toml`
```toml
[input]
    read1 = ['input_read1.fq']
    read2 = ['input_read2.fq'] 
    index1 = ['input_index1.fq']

[[step]]
    action = 'Swap'
    # Should fail - 3 segments but no explicit specification

[output]
    prefix = 'output'
```

#### 4.3 Auto-detection Error Test (Too Few Segments)
**File**: `/project/test_cases/input_validation/swap_auto_detect_too_few_segments/input.toml`
```toml
[input]
    read1 = ['input_read1.fq']

[[step]]
    action = 'Swap'
    # Should fail - only 1 segment

[output]
    prefix = 'output'
```

#### 4.4 Partial Specification Error Test
**File**: `/project/test_cases/input_validation/swap_partial_specification_a_only/input.toml`
```toml
[input]
    read1 = ['input_read1.fq']
    read2 = ['input_read2.fq']

[[step]]
    action = 'Swap'
    segment_a = 'read1'
    # Missing segment_b - should fail

[output]
    prefix = 'output'
```

**File**: `/project/test_cases/input_validation/swap_partial_specification_b_only/input.toml`
```toml
[input]
    read1 = ['input_read1.fq']
    read2 = ['input_read2.fq']

[[step]]
    action = 'Swap'
    segment_b = 'read1'
    # Missing segment_a - should fail

[output]
    prefix = 'output'
```

#### 4.5 Update Existing Test Documentation
Update test files to include expected panic messages:

**File**: `/project/test_cases/input_validation/swap_auto_detect_too_many_segments/expected_panic.txt`
```
Swap requires exactly 2 input segments when segment_a and segment_b are omitted, but 3 segments were provided
```

**File**: `/project/test_cases/input_validation/swap_auto_detect_too_few_segments/expected_panic.txt`
```
Swap requires exactly 2 input segments when segment_a and segment_b are omitted, but 1 segments were provided
```

**File**: `/project/test_cases/input_validation/swap_partial_specification_a_only/expected_panic.txt`
```
Swap requires both segment_a and segment_b to be specified, or both to be omitted for auto-detection with exactly 2 segments
```

**File**: `/project/test_cases/input_validation/swap_partial_specification_b_only/expected_panic.txt`
```
Swap requires both segment_a and segment_b to be specified, or both to be omitted for auto-detection with exactly 2 segments
```

## Implementation Steps

1. **Update Swap struct** - Make segment fields optional
2. **Enhance validation logic** - Add auto-detection and partial specification validation
3. **Create test cases** - Add positive and negative test cases
4. **Update test generator** - Run `dev/update_tests.py`
5. **Run tests** - Verify all tests pass with `cargo test`
6. **Update documentation** - Add usage examples to template.toml or comments

## Validation Criteria

### Success Conditions
- ✅ Auto-detection works with exactly 2 segments
- ✅ Explicit specification still works (backwards compatibility)
- ✅ Validation fails gracefully for edge cases:
  - More than 2 segments without explicit specification
  - Less than 2 segments
  - Partial specification (only one of segment_a/segment_b)
- ✅ All existing tests continue to pass
- ✅ New test cases validate expected behavior

### Error Messages
- Clear, actionable error messages for all failure cases
- Consistent with existing codebase error handling patterns

## Risk Assessment
- **Low Risk**: Changes are additive and maintain backward compatibility
- **Testing Coverage**: Comprehensive test cases cover all edge cases
- **Validation**: Strong input validation prevents invalid configurations

## Documentation Updates
Consider adding usage examples to `/project/src/template.toml` showing both explicit and auto-detection usage patterns.

## Success Metrics
- All existing Swap functionality preserved
- New auto-detection feature works reliably
- Error handling provides clear guidance to users
- Test coverage includes all edge cases
- Performance impact negligible (validation is O(1))
