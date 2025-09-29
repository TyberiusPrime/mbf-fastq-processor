# this needed some refinement. Then Claude only did 80% of the work, 
# but neglected the 'collect the errors' step'

# Error Handling in Processing - Implementation Plan

## Problem Statement

Currently, configuration mistakes that can only be detected during processing (like StoreTagInSequence failing due to missing location data) cause panics that are handled by the "friendly panic" system. This means users don't see the actual error message, making debugging difficult.

The current `apply` method in the `Step` trait returns `(FastQBlocksCombined, bool)` without any error handling mechanism, and panics are the only way to signal errors.

## Current Architecture Analysis

### Key Components

- **Step trait** (`src/transformations.rs:142`): Contains the `apply` method that processes data blocks
- **handle_stage function** (`src/lib.rs:959`): Calls `stage.apply()` in worker threads
- **Processing threads** (`src/lib.rs:676-698`): Worker threads that process data blocks
- **Friendly panic handler** (`src/main.rs:26-31`): Catches panics and shows user-friendly messages

### Specific Problem Case

- **StoreTagInSequence** (`src/transformations/tag/store_tag_in_sequence.rs:59`): Panics when regions lack location data
- Cannot be validated at configuration time, only during processing
- Panic message is lost due to friendly panic handling

## Implementation Plan

### Phase 1: Modify Step Trait Return Type

**Goal**: Change `apply` method to return `Result<(FastQBlocksCombined, bool), ProcessingError>`

**Changes Required**:

1. **Update Step trait** (`src/transformations.rs:142`):
   ```rust
   fn apply(
       &mut self,
       block: crate::io::FastQBlocksCombined,
       input_info: &crate::transformations::InputInfo,
       block_no: usize,
       demultiplex_info: &Demultiplexed,
   ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)>;
   ```

### Phase 2: Update All Step Implementations

**Goal**: Convert all `apply` implementations to return `anyhow::Result` instead of panicking

**Priority Order**:

1. **StoreTagInSequence** - Replace panic with proper error
2. **Other tag operations** - Any other potential panic sites
3. **All remaining steps** - Wrap existing logic in `Ok()`

**StoreTagInSequence specific changes** (`src/transformations/tag/store_tag_in_sequence.rs:31`):

- Replace panic on line 59 with:
  ```rust
  return bail!(
      format!("StoreTagInSequence only works on regions with location data. Region: {region:?}"),
      Some("Set ignore_missing=true to skip regions without location data, or check if location data was lost in previous transformations".to_string())
  );
  ```

### Phase 3: Update Error Propagation in Processing Pipeline

**Goal**: Properly handle and propagate errors from worker threads to main thread

**Changes Required**:

1. **Modify handle_stage function** (`src/lib.rs:946`):

   ```rust
   fn handle_stage(
       // ... existing parameters
   ) -> Result<bool, ProcessingError> {
       // ... existing setup

       let (out_block, stage_continue) = stage.apply(
           out_block,
           input_info,
           block.0,
           // ... existing parameters
       )?; // Propagate error instead of panicking

       // ... rest of function
   }
   ```

2. **Update worker threads** (`src/lib.rs:676-698`):
   - Serial threads: Collect errors and send via error channel
   - Parallel threads: Use error channel to report failures
   - Add error channel alongside data channels

### Phase 4: Error Reporting and Early Termination

**Goal**: Show meaningful error messages to users and gracefully terminate processing

**Changes Required**:

1. **Error channel setup** (`src/lib.rs`):

   - Create error channel alongside existing channels
   - Monitor error channel in main thread
   - Implement graceful shutdown on first error

2. **Error display** (`src/main.rs`):

   - Show specific error message instead of generic panic message
   - Include suggestions when available

3. **Thread coordination**:
   - Threads learn about processing stop by their channels being closed.
   - Wait for threads to complete current blocks
   - Ensure clean resource cleanup

### Phase 5: Testing and Validation

**Goal**: Ensure error handling works correctly and doesn't break existing functionality

**Test Cases to Add**:

1. **StoreTagInSequence error conditions**:

   - Missing location data triggers proper error
   - Error message includes helpful suggestion
   - Processing stops cleanly

2. **Backwards compatibility**:
   - All existing test cases still pass
   - Performance impact is minimal
   - No functional regressions

**Testing Strategy**:

1. Run `dev/update_tests.py` followed by `cargo test` after each phase
2. Add specific test cases for error conditions
3. Test with real-world configurations that trigger errors
4. Verify error messages are user-friendly and actionable

### Phase 6: Documentation and Error Message Improvements

**Goal**: Ensure users can understand and fix configuration errors

**Changes Required**:

1. Improve error messages with specific guidance
2. Add configuration validation where possible
3. Update documentation with error handling information
4. Add examples of common error scenarios and fixes

# Success Criteria

1. ✅ Users see specific error messages instead of generic panic messages
2. ✅ Processing terminates gracefully on configuration errors
3. ✅ Error messages include actionable suggestions
4. ✅ No performance regression in normal processing
5. ✅ All existing tests continue to pass
6. ✅ New tests cover error conditions

## Risk Mitigation

- **Large refactor risk**: Implement incrementally, test after each phase
- **Performance risk**: Benchmark before/after, optimize if needed
- **Compatibility risk**: Maintain existing API where possible
- **Testing complexity**: Use existing test infrastructure, add specific error tests

This plan transforms the error handling from panic-based to proper error propagation, giving users actionable feedback when configuration issues are detected during processing.
