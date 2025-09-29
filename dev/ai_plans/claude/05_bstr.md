# Replace Vec<u8> DNA Representations with bstr

## Objective
Replace all `Vec<u8>` DNA sequence representations in the codebase with the `bstr` crate types (`BString` and `BStr`) to improve debugging experience and readability. Currently, Vec<u8> sequences print as raw byte arrays like `[65, 84, 67, 71]` instead of readable strings like `"ATCG"`.

## Analysis Summary

### Current Vec<u8> Usage for DNA Sequences
Based on codebase analysis, the main Vec<u8> usage for DNA sequences is in:

1. **Core DNA Types (`src/dna.rs`)**:
   - `Hit.sequence: Vec<u8>` - stores matched DNA sequences
   - `joined_sequence() -> Vec<u8>` - combines multiple hit sequences  
   - `reverse_complement_iupac() -> Vec<u8>` - returns reversed complement sequences

2. **Transformation Steps**:
   - Tag extraction and replacement sequences
   - Barcode sequences for demultiplexing
   - Filter sequences for exact matching
   - Edit operations (insert, delete, replace sequences)
   - Quality score representations (also Vec<u8>)

3. **I/O Layer (`src/io.rs`)**:
   - FastQ sequence and quality data storage
   - Temporary buffers for sequence manipulation

### Benefits of bstr Migration

1. **Improved Debugging**: DNA sequences will display as readable strings like `"ATCG"` instead of `[65, 84, 67, 71]`
2. **Better Error Messages**: When debugging sequence mismatches or processing errors
3. **Maintained Performance**: bstr wraps Vec<u8> with zero-cost abstractions
4. **UTF-8 Safety**: Handles non-UTF-8 bytes gracefully (important for quality scores)

## Implementation Plan

### Phase 1: Add bstr Dependency
- Add `bstr = "1.10"` to Cargo.toml dependencies
- Add `use bstr::{BStr, BString, ByteSlice}` imports where needed

### Phase 2: Core DNA Types (src/dna.rs)
- Change `Hit.sequence: Vec<u8>` → `Hit.sequence: BString`
- Update `joined_sequence()` → return `BString` 
- Update `reverse_complement_iupac()` → return `BString`
- Update all function signatures that accept/return Vec<u8> for DNA sequences
- Update test assertions to work with bstr types

### Phase 3: Configuration and Deserialization (src/config/)
- Update DNA sequence deserialization functions to return `BString`
- Modify `dna_from_string`, `iupac_from_string` in `src/config/deser.rs`
- Update barcode parsing to use `BString`

### Phase 4: Transformation Steps
- **Filters**: Update exact sequence matching to use `BString`
- **Tags**: Update tag search, replacement, and separator sequences
- **Demultiplex**: Update barcode sequences to `BString`
- **Edits**: Update insert/replace sequences to `BString`
- **Reports**: Update sequence collectors to use `BString`

### Phase 5: I/O Layer (src/io.rs)
- Update FastQ sequence storage (careful analysis needed for performance)
- Consider keeping quality scores as Vec<u8> since they're not DNA sequences
- Update sequence replacement and manipulation methods

### Phase 6: Testing and Validation
- Run full test suite to ensure no regressions
- Verify improved debug output in integration tests
- Performance benchmarking to ensure no significant slowdowns
- Update any hardcoded sequence expectations in tests

## Migration Strategy

### Incremental Approach
1. Start with core DNA types that don't affect I/O performance
2. Use `BString::from()` and `.into()` conversions during transition
3. Update one module at a time with comprehensive testing
4. Keep quality scores as `Vec<u8>` initially (they're not DNA, just bytes)

### Backward Compatibility
- Use trait bounds like `AsRef<[u8]>` in function signatures where possible
- This allows both `BString` and `Vec<u8>` to be passed during transition
- Gradually tighten types as migration progresses

### Risk Mitigation
- Focus on DNA sequence data only, not all Vec<u8> usage
- Preserve performance-critical paths (I/O buffers) until final phase
- Extensive testing at each phase
- Git commits after each successful module migration

## Expected Impact

### Positive
- **Significantly improved debugging experience** - main goal achieved
- Better error messages and logging
- More intuitive code when working with DNA sequences
- Zero runtime performance cost

### Considerations
- Requires careful handling of the transition period
- Some API changes for functions accepting/returning DNA sequences
- Need to distinguish between DNA sequences and raw bytes (quality scores)
- Compilation time may slightly increase due to additional trait implementations

## Timeline Estimate
- Phase 1: 30 minutes (dependency addition)
- Phase 2: 2-3 hours (core DNA types + tests)  
- Phase 3: 1 hour (config/deserialization)
- Phase 4: 3-4 hours (transformation steps)
- Phase 5: 2-3 hours (I/O layer - most complex)
- Phase 6: 1-2 hours (comprehensive testing)

**Total: ~10-14 hours** for complete migration

## Success Criteria
- All tests pass
- Debug output shows readable DNA sequences instead of byte arrays
- No performance regression on benchmarks
- All DNA-related Vec<u8> converted to BString
- Quality scores remain as Vec<u8> (they're not DNA sequences)