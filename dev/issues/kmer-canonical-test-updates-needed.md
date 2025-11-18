# Test Updates Needed for Canonical Kmer Changes

## Summary

The canonical kmer implementation has been changed to use true canonical kmers (lexicographically smaller orientation) instead of storing both forward and reverse complement. This changes the behavior and output of tests using `count_reverse_complement=true` or `canonical=true`.

## Required Actions

### 1. Regenerate Test Outputs

Run the test update script to regenerate expected outputs:

```bash
dev/update_generated.sh
# or individually:
python dev/_update_tests.py
python dev/_update_cookbooks.py
```

### 2. Verify Tests Pass

After regenerating outputs, run the full test suite:

```bash
cargo test
```

### 3. Review Changed Test Files

The following test cases use canonical kmers and will have different outputs:

#### Integration Tests
- `test_cases/single_step/calc/kmer/phix/` - Uses both canonical and non-canonical modes
  - **Expected change**: `output_kmer.tsv` will have different counts since canonical mode now properly accumulates both orientations
  - The test specifically compares `kmer_count_canonical` vs `kmer_count_non_canonical`

#### Cookbooks
- `cookbooks/04-phiX-removal/` - Uses canonical=true for PhiX detection
  - **Expected change**: `reference_output/output_without_phix_kmer_analysis.tsv` will show different kmer counts
  - Kmer counts should still correctly identify PhiX reads (might actually be more accurate now)

### 4. What Changed

**Old behavior** (`count_reverse_complement=true`):
- Database stored BOTH forward and reverse complement (2x entries)
- Query matched either orientation by lookup
- Memory usage: ~2x unique kmers
- min_count filtering applied to each orientation separately

**New behavior** (`count_reverse_complement=true`):
- Database stores only CANONICAL form (1x entries)
- Query converts to canonical before lookup
- Memory usage: ~1x unique kmers (50% reduction)
- min_count filtering applied to combined count

**Expected changes in test outputs:**
1. Kmer counts in canonical mode should be SAME or HIGHER (counts accumulate)
2. Database memory usage reduced by ~50%
3. Matching behavior unchanged (both orientations still match)

### 5. Special Case: PhiX Test

The phix test (`test_cases/single_step/calc/kmer/phix/`) includes:
- Read `2357_TTAAGA-phix-reverse`: This is a reverse-complement PhiX read
- Old output: `kmer_count_non_canonical = 0` (expected, no forward match)
- Old output: `kmer_count_canonical = 32` (expected, reverse matches)

**Expected new output:**
- `kmer_count_non_canonical = 0` (unchanged, still no forward match)
- `kmer_count_canonical = 32` (unchanged, canonical matching works correctly)

The counts should remain the same because the functionality is equivalent - we still match both orientations, just more efficiently.

### 6. Potential Issues to Watch For

If tests fail after regeneration:
1. **Counts decreased**: Bug in canonical implementation - both orientations should still match
2. **Counts increased dramatically**: Check if we're double-counting somehow
3. **Non-canonical tests failed**: Those shouldn't be affected, investigate carefully

### 7. Manual Verification

To manually verify the canonical kmer behavior is correct:

```bash
# Run one of the kmer tests
cd test_cases/single_step/calc/kmer/phix
../../../../target/release/mbf-fastq-processor input.toml

# Check the output
cat output_kmer.tsv

# Verify:
# - PhiX reads (containing "phix" in name) should have kmer_count_canonical ~= 32
# - Reverse PhiX reads should ALSO match (this is the key test)
# - Non-PhiX reads should have kmer_count_canonical = 0
```

## Implementation Status

- [x] Core canonical kmer implementation
- [x] Unit tests added
- [x] Documentation updated
- [ ] Integration test outputs regenerated
- [ ] Cookbook outputs regenerated
- [ ] All tests verified passing

## Notes

Due to build environment issues (linker problems), tests couldn't be run during implementation. The code changes are complete and unit tests are in place, but integration test expectations need to be regenerated and verified in a proper build environment.
