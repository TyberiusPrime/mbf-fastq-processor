# AI Plan 16: Reorganize Transformation Structs into Individual Files

## Goal
Place each Transformation struct (implements Step trait) into its own file while maintaining the current high-level structure (filters/, edits/, tag/, etc.). This will improve code organization, maintainability, and make it easier to locate specific transformations.

## Current State Analysis
Currently, transformation structs are organized in 6 main files:
- `src/transformations/filters.rs` - 11 structs implementing Step
- `src/transformations/edits.rs` - 14 structs implementing Step  
- `src/transformations/tag.rs` - 25 structs implementing Step
- `src/transformations/reports.rs` - 9 structs implementing Step (plus support structs)
- `src/transformations/validation.rs` - 2 structs implementing Step
- `src/transformations/demultiplex.rs` - 1 struct implementing Step

Total: **62 transformation structs** to be reorganized.

## Planned Directory Structure
```
|── src/transformations.rs (updated to reference new structure)
|── src/transformations/
│├── filters.rs
│├── filters/
││   ├── Head.rs
││   ├── Skip.rs
││   ├── Empty.rs
││   ├── QualifiedBases.rs
││   ├── TooManyN.rs
││   ├── LowComplexity.rs
││   ├── Sample.rs
││   ├── Duplicates.rs
││   ├── OtherFileByName.rs
││   ├── OtherFileBySequence.rs
││   └── FilterByNumericTag.rs
│├── edits.rs
│├── edits/
││   ├── CutStart.rs
││   ├── CutEnd.rs
││   ├── MaxLen.rs
││   ├── Prefix.rs
││   ├── Postfix.rs
││   ├── ReverseComplement.rs
││   ├── Phred64To33.rs
││   ├── Rename.rs
││   ├── TrimQualityStart.rs
││   ├── TrimQualityEnd.rs
││   ├── SwapR1AndR2.rs
││   ├── LowercaseTag.rs
││   ├── UppercaseTag.rs
││   ├── LowercaseSequence.rs
││   └── UppercaseSequence.rs
│├── tag.rs/
│├── tag/
││   ├── ExtractIUPAC.rs
││   ├── ExtractRegex.rs
││   ├── ExtractAnchor.rs
││   ├── ExtractPolyTail.rs
││   ├── ExtractIUPACSuffix.rs
││   ├── FilterByTag.rs
││   ├── TrimAtTag.rs
││   ├── ExtractRegion.rs
││   ├── ExtractRegions.rs
││   ├── StoreTagInSequence.rs
││   ├── StoreTagInComment.rs
││   ├── StoreTaglocationInComment.rs
││   ├── ExtractLength.rs
││   ├── ExtractMeanQuality.rs
││   ├── ExtractGCContent.rs
││   ├── ExtractNCount.rs
││   ├── ExtractLowComplexity.rs
││   ├── ExtractQualifiedBases.rs
││   ├── RemoveTag.rs
││   ├── StoreTagsInTable.rs
││   ├── QuantifyTag.rs
││   ├── ExtractRegionsOfLowQuality.rs
││   └── ReplaceTagWithLetter.rs
│├── reports.rs
│├── reports/
││   ├── Progress.rs
││   ├── Report.rs
││   ├── ReportCount.rs
││   ├── ReportLengthDistribution.rs
││   ├── ReportDuplicateCount.rs
││   ├── ReportDuplicateFragmentCount.rs
││   ├── ReportBaseStatisticsPart1.rs
││   ├── ReportBaseStatisticsPart2.rs
││   ├── ReportCountOligos.rs
││   └── Inspect.rs
│├── validation.rs
│├── validation/
││   ├── ValidateSeq.rs
││   └── ValidatePhred.rs
│├── demultiplex.rs
```

## Execution Plan

### Phase 1: Create Directory Structure
1. Create subdirectories: `filters/`, `edits/`, `tag/`, `reports/`, `validation/`, `demultiplex/`

### Phase 2: Extract Filter Transformations (11 structs)
2. Extract each struct from `filters.rs` into individual files:
   - Head → `filters/Head.rs`
   - Skip → `filters/Skip.rs`
   - Empty → `filters/Empty.rs`
   - QualifiedBases → `filters/QualifiedBases.rs`
   - TooManyN → `filters/TooManyN.rs`
   - LowComplexity → `filters/LowComplexity.rs`
   - Sample → `filters/Sample.rs`
   - Duplicates → `filters/Duplicates.rs`
   - OtherFileByName → `filters/OtherFileByName.rs`
   - OtherFileBySequence → `filters/OtherFileBySequence.rs`
   - FilterByNumericTag → `filters/FilterByNumericTag.rs`
3. Create `filters.rs` with proper module declarations and re-exports

### Phase 3: Extract Edit Transformations (14 structs)
4. Extract each struct from `edits.rs` into individual files:
   - CutStart → `edits/CutStart.rs`
   - CutEnd → `edits/CutEnd.rs`
   - MaxLen → `edits/MaxLen.rs`
   - Prefix → `edits/Prefix.rs`
   - Postfix → `edits/Postfix.rs`
   - ReverseComplement → `edits/ReverseComplement.rs`
   - Phred64To33 → `edits/Phred64To33.rs`
   - Rename → `edits/Rename.rs`
   - TrimQualityStart → `edits/TrimQualityStart.rs`
   - TrimQualityEnd → `edits/TrimQualityEnd.rs`
   - SwapR1AndR2 → `edits/SwapR1AndR2.rs`
   - LowercaseTag → `edits/LowercaseTag.rs`
   - UppercaseTag → `edits/UppercaseTag.rs`
   - LowercaseSequence → `edits/LowercaseSequence.rs`
   - UppercaseSequence → `edits/UppercaseSequence.rs`
5. Create `edits.rs` with proper module declarations and re-exports

### Phase 4: Extract Tag Transformations (25 structs)
6. Extract each struct from `tag.rs` into individual files:
   - ExtractIUPAC → `tag/ExtractIUPAC.rs`
   - ExtractRegex → `tag/ExtractRegex.rs`
   - ExtractAnchor → `tag/ExtractAnchor.rs`
   - ExtractPolyTail → `tag/ExtractPolyTail.rs`
   - ExtractIUPACSuffix → `tag/ExtractIUPACSuffix.rs`
   - FilterByTag → `tag/FilterByTag.rs`
   - TrimAtTag → `tag/TrimAtTag.rs`
   - ExtractRegion → `tag/ExtractRegion.rs`
   - ExtractRegions → `tag/ExtractRegions.rs`
   - StoreTagInSequence → `tag/StoreTagInSequence.rs`
   - StoreTagInComment → `tag/StoreTagInComment.rs`
   - StoreTaglocationInComment → `tag/StoreTaglocationInComment.rs`
   - ExtractLength → `tag/ExtractLength.rs`
   - ExtractMeanQuality → `tag/ExtractMeanQuality.rs`
   - ExtractGCContent → `tag/ExtractGCContent.rs`
   - ExtractNCount → `tag/ExtractNCount.rs`
   - ExtractLowComplexity → `tag/ExtractLowComplexity.rs`
   - ExtractQualifiedBases → `tag/ExtractQualifiedBases.rs`
   - RemoveTag → `tag/RemoveTag.rs`
   - StoreTagsInTable → `tag/StoreTagsInTable.rs`
   - QuantifyTag → `tag/QuantifyTag.rs`
   - ExtractRegionsOfLowQuality → `tag/ExtractRegionsOfLowQuality.rs`
   - ReplaceTagWithLetter → `tag/ReplaceTagWithLetter.rs`
7. Create `tag.rs` with proper module declarations and re-exports

### Phase 5: Extract Report Transformations (9 main structs + support structs)
8. Extract each struct from `reports.rs` into individual files:
   - Progress → `reports/Progress.rs`
   - Report → `reports/Report.rs`
   - _ReportCount → `reports/ReportCount.rs`
   - _ReportLengthDistribution → `reports/ReportLengthDistribution.rs`
   - _ReportDuplicateCount → `reports/ReportDuplicateCount.rs`
   - _ReportDuplicateFragmentCount → `reports/ReportDuplicateFragmentCount.rs`
   - _ReportBaseStatisticsPart1 → `reports/ReportBaseStatisticsPart1.rs`
   - _ReportBaseStatisticsPart2 → `reports/ReportBaseStatisticsPart2.rs`
   - _ReportCountOligos → `reports/ReportCountOligos.rs`
   - Inspect → `reports/Inspect.rs`
9. Handle support structs (ReportData, PerReadReportData, etc.) - may need to be in shared module or duplicated
10. Create `reports.rs` with proper module declarations and re-exports

### Phase 6: Extract Validation Transformations (2 structs)
11. Extract each struct from `validation.rs` into individual files:
    - ValidateSeq → `validation/ValidateSeq.rs`
    - ValidatePhred → `validation/ValidatePhred.rs`
12. Create `validation.rs` with proper module declarations and re-exports

### Phase 7: Extract Demultiplex Transformation (1 struct)
13. Extract struct from `demultiplex.rs` into individual file:
    - Demultiplex → `demultiplex/Demultiplex.rs`
14. Create `demultiplex.rs` with proper module declarations and re-exports

### Phase 8: Update Module System
15. Update `src/transformations.rs` to reference new modular structure
16. Update all imports throughout the codebase to reference new module paths

### Phase 9: Testing and Validation
17. Run `dev/update_tests.py` to update test artifacts
18. Run `cargo test` to ensure all tests pass
19. Run `cargo check` and `cargo clippy` to ensure no compilation or linting issues
20. Run `cargo build --release` to ensure clean release build

## Key Considerations

### Import Management
- Each individual file will need to import common dependencies (Step trait, anyhow, etc.)
- Shared utilities and types may need to be moved to common modules
- Support structs in reports.rs may need special handling (shared module or duplication)

### Code Extraction Strategy
- Extract struct definition, impl blocks, and any private helper functions
- Preserve all documentation and comments
- Maintain exact same functionality and behavior
- Handle complex dependencies between structs (especially in reports.rs)

### Module Re-exports
- Each `mod.rs` will re-export all structs to maintain same public API
- Main `transformations/mod.rs` will need updating to reference new structure
- Ensure backward compatibility for all existing imports

### Testing Concerns
- All existing tests should continue to work without modification
- No behavioral changes - purely structural reorganization
- Must run full test suite to validate

## Expected Benefits
- **Improved Navigation**: Each transformation in its own file makes it easier to find and edit
- **Better IDE Support**: Faster file loading, better symbol navigation
- **Cleaner Git History**: Changes to individual transformations won't affect other transformations
- **Easier Maintenance**: Smaller files are easier to understand and modify
- **Scalability**: Easy to add new transformations without growing monolithic files

## Risk Mitigation
- Thorough testing after each phase to catch any issues early
- Careful handling of shared dependencies and support structs
- Incremental approach allows rollback if issues are discovered
- Preserving exact functionality to avoid breaking existing functionality

## Files to be Modified
- 62 new individual transformation files
- 6 new `mod.rs` files for subdirectories
- 1 updated `src/transformations/mod.rs`
- Potential updates to import statements throughout codebase
- 6 original files will be removed after extraction
