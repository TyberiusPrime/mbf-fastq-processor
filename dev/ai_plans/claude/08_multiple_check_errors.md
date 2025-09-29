# Plan: Implement Multiple Error Collection in Config::check

## Current Problem Config::check currently aborts on the first error it
encounters, which means users only see one validation error at a time. This
creates a poor user experience where they have to fix one error, re-run, and
then discover the next error.

## Goal
Modify Config::check to collect ALL validation errors and return them together, allowing users to see and fix all configuration issues in one pass.

## Implementation Plan

### 1. Analyze Current Config::check Implementation
- [ ] Locate Config::check method in the codebase
- [ ] Understand current error handling pattern
- [ ] Identify all validation points that can fail
- [ ] Document current return type and error structure

### 2. Design Error Collection Strategy
- [ ] Decide on error collection approach (Vec<Error>, custom error type, etc.)
- [ ] Determine how to preserve error context and location information
- [ ] Plan how to handle errors that would prevent further validation

### 3. Refactor Error Handling
- [ ] Replace early returns with error collection
- [ ] Modify validation logic to continue after encountering errors
- [ ] Ensure critical errors still prevent unsafe operations
- [ ] Update return type to handle multiple errors

### 4. Update Error Display
- [ ] Modify error formatting to handle multiple errors cleanly
- [ ] Ensure error messages remain clear and actionable
- [ ] Test error output formatting with multiple errors

### 5. Test Implementation
- [ ] Run existing tests to ensure no regressions
- [x] Create test cases with multiple validation errors (two_mistakes_post_deserialization)
- [ ] Verify all errors are collected and displayed properly
- [ ] Test edge cases (no errors, single error, many errors)

### 6. Final Validation
- [ ] Run cargo clippy to check for code quality issues
- [ ] Run cargo test to ensure all tests pass
- [ ] Test with real configuration files that have multiple errors
