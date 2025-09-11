# Plan 15: Automated Test for Target Examples Consistency

## Objective Create an automated test that ensures all target examples in
documentation files and template.toml contain the correct pattern: `target =
'Read1' # Read2|Index1|Index2` (with or without `|All`).

## Problem Statement
Currently, documentation files and template.toml contain target examples that should follow a consistent comment pattern showing all available options. We need to verify that:
1. All target lines in template.toml follow the correct pattern
2. All target lines in documentation files follow the correct pattern
3. The pattern correctly includes all valid target options based on the transformation's capabilities

## Analysis of Current State

### Template.toml Patterns Found
From examination of template.toml, target examples currently use patterns like:
- `target = "Read1" # |"Read2"|"Index1"|"Index2"|"All"`
- `target = "Read1" #|"Read2"|"Index1"|"Index2"|"All"`
- `target = "Read1" # |"Read2"|"Index1"|"Index2"`

### Documentation Patterns Found
From ExtractLength.md:
- `target = "read1" # Any of your input segments`

### Inconsistencies Identified
1. Quote style varies: single quotes vs double quotes
2. Spacing around pipe separators varies
3. Some include "All", others don't
4. Comment format varies (with/without spaces, different quote styles)

## Implementation Plan

### Phase 1: Extend Existing Documentation Test Structure
Build upon the existing test in tests/template_and_documentation_verification.rs,
add in code to map each transformation to its valid target options.

### Phase 2: Define Target Patterns
Create standardized search pattern for valid target examples:

1. **Targets Pattern**: `target = "read1" # Any of your input segments`
1. **TargetsPlusAll Pattern**: `target = "read1" # Any of your input segments, or 'All'`

### Phase 3: Create Test Function
Implement `test_target_examples_consistency()` that:

1. **Template.toml Verification**:
   - extend test_every_step_has_a_template_section() to also check for the presence of the appropriate pattern.

2. **Documentation Files Verification**:
   - in test_documentation_toml_examples_parse(),  check that each toml block contains the appropriate pattern.

### Phase 5: Error Reporting
Provide detailed error messages:
- Filename / section that failed.

### Phase 6: Commit at this point

### Phase 7: 
Run the extend test case, follow it's bread crumbs to fix all instances.