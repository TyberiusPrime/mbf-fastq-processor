# AI Plan 12: Fix Compression Level Propagation from Output/Inspect to Writers

## Problem Analysis
Currently, both the main output section and the Inspect report step have `compression_level` fields (Option<u8> for output, Option<u32> for Inspect) that are parsed from TOML configs but never actually used. The HashedAndCompressedWriter in output.rs hardcodes compression levels:
- Gzip: uses `flate2::Compression::default()` (level 6)
- Zstd: hardcoded to level 5
- The compression_level fields in Output and Inspect structs are ignored

## Implementation Plan

### 1. Update HashedAndCompressedWriter (src/output.rs)
- Add `compression_level: Option<u32>` parameter to `new()` method
- Modify Gzip branch to use `flate2::Compression::new(level)` when level provided
- Modify Zstd branch to use provided level instead of hardcoded 5
- Use sensible defaults when no level provided (6 for gzip, 3 for zstd)

### 2. Update OutputFile Creation (src/lib.rs) 
- Modify `new_file()` method to accept compression_level parameter
- Modify `new_stdout()` method to accept compression_level parameter  
- Pass the output.compression_level from config to all OutputFile::new_file() calls
- Update all call sites in the main processing loop

### 3. Update Inspect Implementation (src/transformations/reports.rs)
- Pass self.compression_level to HashedAndCompressedWriter::new() in finalize() method
- Already has the field, just need to wire it through

### 4. Type Consistency
- Standardize compression_level type to Option<u32> (change Output.compression_level from Option<u8> to Option<u32>)
- Update template.toml and documentation to reflect proper ranges

### 5. Testing Strategy
- Create test case for main output with custom compression levels
- Create test case for Inspect with custom compression levels
- Test both gzip and zstd compression with different levels
- Verify compressed output actually uses specified levels (file size comparison)
- Ensure existing tests still pass with default compression levels

### 6. Documentation Updates
- Update src/template.toml with proper compression level examples
- Update docs/content/docs/reference/Output Section.md 
- Update docs/content/docs/reference/Report steps/Inspect.md

## Files to Modify
- src/output.rs (HashedAndCompressedWriter::new signature and implementation)
- src/lib.rs (OutputFile::new_file, new_stdout methods and all call sites)
- src/config/mod.rs (Output.compression_level type change u8->u32)
- src/transformations/reports.rs (wire compression_level through to writer)
- src/template.toml (documentation)
- docs/content/docs/reference/Output Section.md
- docs/content/docs/reference/Report steps/Inspect.md

## Testing Plan
- Add test_cases/output_compression_levels/ with gzip level 9
- Add test_cases/inspect_compression_levels/ with zstd level 1  
- Run full test suite to ensure no regressions
- Manual verification that different compression levels produce different file sizes

## Implementation Status
- [x] Create AI Plan 12 file
- [x] Update HashedAndCompressedWriter::new() signature and implementation  
- [x] Update OutputFile methods to accept compression_level
- [x] Change Output.compression_level type to Option<u32>
- [x] Update all OutputFile::new_file() call sites
- [x] Wire compression_level through in Inspect.finalize()
- [x] Create test cases for output compression levels
- [x] Create test cases for inspect compression levels
- [x] Run full test suite and fix regressions
- [x] Update documentation files

## Completed Successfully ✅

The compression level propagation has been successfully implemented and tested. Users can now specify `compression_level` in both the main `[output]` section and individual `[[step]]` Inspect sections to control the compression level of their output files.

### Key Changes Made:
1. **HashedAndCompressedWriter**: Now accepts and uses `compression_level` parameter
2. **OutputFile methods**: Updated to pass through compression levels  
3. **Type consistency**: Standardized to Option<u32> for all compression levels
4. **Inspect step**: Now properly uses compression_level setting
5. **Backward compatibility**: All existing configs continue to work with default levels
6. **Testing**: Added comprehensive test cases demonstrating the functionality
7. **Documentation**: Updated template.toml and markdown docs with proper examples

### Default Compression Levels:
- **Gzip**: Level 6 (matches flate2 default)
- **Zstd**: Level 5 (maintains previous hardcoded behavior)