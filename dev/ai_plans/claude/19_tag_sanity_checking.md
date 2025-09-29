# This one worked.

# Tag Sanity Checking - Implementation Plan

## Goal
Implement tag output type declarations and validation to prevent runtime panics when tag filters encounter unexpected tag types. Tags will declare their output type (location, numeric, bool), and the three tag filters (FilterByTag, FilterByNumericTag, FilterByBoolTag) will validate tag types during step validation.

## Current Analysis
- Three tag filters exist: `FilterByTag`, `FilterByNumericTag`, `FilterByBoolTag`
- All currently panic at runtime when encountering wrong tag types:
  - `FilterByBoolTag`: panics with "FilterByBoolTag applied to non-boolean tag"  
  - `FilterByNumericTag`: silently filters out non-numeric values (line 56: `false`)
  - `FilterByTag`: expects tag existence, works with any type
- Tag system uses `TagValue` enum with `Missing`, `Sequence(Hits)`, `Numeric(f64)`, `Bool(bool)`
- Steps implement `Step` trait with `uses_tags()`, `sets_tag()`, `removes_tag()` methods

## Implementation Steps

### 1. Add Tag Type Declaration to Step Trait
- Add `declares_tag_type(&self) -> Option<(String, TagValueType)>` method to `Step` trait in `src/transformations.rs`
- Create `TagValueType` enum: `Location`, `Numeric`, `Bool`
- Default implementation returns `None`

### 2. Update All Tag-Setting Steps  
- Find all steps that implement `sets_tag()` and add `declares_tag_type()` implementation
- Tag extraction steps (ExtractLength, ExtractQualifiedBases, etc.) → `Numeric`
- Boolean tag steps (filtering results, etc.) → `Bool`
- Sequence extraction steps (ExtractRegex, ExtractAnchor, etc.) → `Location`

### 3. Implement Tag Type Validation
- Add validation method `validate_tag_types()` to check tag type compatibility
- Call this in the main validation pipeline before processing starts
- For each filter step using tags:
  - `FilterByNumericTag` requires `Numeric` tags
  - `FilterByBoolTag` requires `Bool` tags  
  - `FilterByTag` accepts any non-missing tag type

### 4. Update Filter Step Validation
- Modify `FilterByNumericTag::validate()` to check upstream tag types
- Modify `FilterByBoolTag::validate()` to check upstream tag types
- Replace runtime panics with validation-time errors

### 5. Error Messages
- Create clear validation error messages like:
  - "FilterByNumericTag step 'filter_length' expects numeric tag 'length', but upstream step 'extract_bool' declares bool tag"
  - "FilterByBoolTag step 'filter_pass' expects bool tag 'passes', but no upstream step declares tag 'passes'"

### 6. Testing
- Add test cases for tag type mismatches in `test_cases/extraction/`
- Test each filter type with wrong tag types
- Ensure validation catches issues before processing
- Update existing tests if needed with `dev/update_tests.py`

## Files to Modify

### Core Implementation
- `src/transformations.rs`: Add `TagValueType` enum and `declares_tag_type()` to `Step` trait
- `src/transformations/filters/by_numeric_tag.rs`: Add type validation
- `src/transformations/filters/by_bool_tag.rs`: Add type validation, remove panic
- `src/transformations/filters/by_tag.rs`: Update validation (if needed)

### Tag Declaration Updates
- All files in `src/transformations/extract/`: Add `declares_tag_type()` implementations
- All files in `src/transformations/tag/`: Add `declares_tag_type()` implementations  
- All files in `src/transformations/reports/`: Add `declares_tag_type()` for steps that set tags

### Validation Pipeline  
- `src/lib.rs` or main validation code: Add tag type validation to pipeline setup

### Tests
- Add new test cases in `test_cases/extraction/` for type validation failures
- Run `dev/update_tests.py` after adding test cases

## Expected Outcome
- No more runtime panics from tag type mismatches
- Clear validation errors at configuration time
- Type safety for the tag system
- Better developer experience with early error detection
