# outcome: success
# Plan: Tests for Reference Documentation

## Objective
Implement comprehensive tests for the reference documentation section similar to the existing template tests. The goal is to ensure documentation completeness and correctness by verifying:

1. Every transformation has a corresponding documentation file
2. TOML code examples in documentation files parse correctly as valid configurations

## Current State Analysis

### Existing Template Testing (in `tests/template_parsing.rs`)
- Extracts all transformations from `src/transformations.rs` enum
- Verifies each transformation has a section in `src/template.toml`
- Validates that extracted TOML sections parse correctly with the config system
- Uses regex to find transformation names and extract sections

### Documentation Structure
- Reference docs located in `docs/content/docs/reference/`
- Organized in categories: Filter steps, Modification steps, Report steps, Tag steps, Validation Steps
- Each transformation has its own `.md` file with TOML examples
- TOML examples are embedded in markdown code blocks with `toml` language tag

## Implementation Plan

### Phase 1: Documentation Discovery System
Create functions to:
- **`get_all_doc_files()`**: Scan `docs/content/docs/reference/` recursively for `.md` files
- **`extract_transformation_from_filename()`**: Convert file paths to expected transformation names
  - Example: `Filter steps/FilterMinLen.md` → `FilterMinLen`
- **`extract_toml_from_markdown()`**: Parse markdown files to extract TOML code blocks

### Phase 2: Completeness Verification
Implement test: **`test_every_transformation_has_documentation()`**
- Get all transformations from `src/transformations.rs` (reuse existing function)
- Check that each transformation has a corresponding documentation file
- Report missing documentation files
- Skip internal transformations (those starting with `_`)

### Phase 3: TOML Example Validation
Implement test: **`test_documentation_toml_examples_parse()`**
- For each documentation file, extract all TOML code blocks
- Wrap each TOML snippet in a minimal valid configuration structure:
  ```toml
  [input]
  read1 = "test_r1.fastq"
  read2 = "test_r2.fastq"

  [output]
  prefix = "output"
  format = "raw"
  report_json = false  # or true if Report action
  report_html = false

  # EXTRACTED_TOML_GOES_HERE
  ```
- Handle special cases like tag-based transformations (add prerequisite ExtractRegion step)
- Parse with `mbf_fastq_processor::config::Config::from_str()`
- Validate with `config.check()`

### Phase 4: Integration and Error Reporting
- Create comprehensive error messages showing which files fail and why
- Handle edge cases:
  - Multiple TOML blocks in one file
  - Files without TOML examples (error)
  - Malformed markdown structure

## File Structure
```
tests/
├── template_parsing.rs          # rename to 'template_and_documentation_verification.rs
```

## Expected Test Functions
1. `test_every_transformation_has_documentation()` - Completeness check
2. `test_documentation_toml_examples_parse()` - TOML validation
3. `get_all_doc_files()` - Helper function
4. `extract_transformation_from_filename()` - Helper function  
5. `extract_toml_from_markdown()` - Helper function
6. `wrap_toml_in_config()` - inline 


## Order of operations
- write the test code
- commit 
- then loop cargo test until every transformation passes, fixing the documentation
(by following the examples in templates.toml). Commit after each fix.

## Success Criteria
- All transformations have documentation files
- All TOML examples in documentation parse as valid configurations
- Clear error reporting when documentation is missing or invalid
- Tests run as part of `cargo test` suite
- Documentation quality gate prevents regressions

## Benefits
- Ensures documentation stays in sync with code changes
- Validates that user-facing examples actually work
- Improves confidence in documentation accuracy
- Catches documentation regressions in CI/CD pipeline
