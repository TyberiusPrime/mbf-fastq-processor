# this one worked half... 
# the code was good, the test case follow through wasn't

# Hamming Correction Step Implementation Plan

## Context
Currently, hamming correction is embedded in the demultiplex step (src/transformations/demultiplex.rs:107-127). We need to extract this into a standalone step that can correct barcodes/tags and then be used by demultiplex or other transformations.

## Current Implementation Analysis
- Hamming correction is in `demultiplex.rs` apply() method
- Uses two algorithms:
  - IUPAC hamming distance for barcodes with ambiguous nucleotides (lines 107-115)  
  - Standard hamming distance for exact matches (lines 116-126)
- Correction happens when exact barcode match fails
- Uses `max_hamming_distance` parameter
- Replaces tag with corrected barcode if within distance threshold

## Implementation Plan

### Phase 1: Add [barcodes.<name>] Configuration Section

**Step 1.1: Extend TOML configuration structure**
- Add `[barcodes.<name>]` section to config parsing in `src/config/`
- Create new `Barcodes` struct to hold barcode mappings
- Update config validation to ensure barcode names are unique
- Add tests for new configuration parsing

**Step 1.2: Update global configuration struct**
- Modify main config struct to include optional barcodes section
- No backwards compatibility, we just adjust the test cases.

### Phase 2: Modify Demultiplex to Use Shared Barcodes

**Step 2.1: Update Demultiplex struct**
- Remove `barcode_to_name` 
- Add `barcodes` field to reference shared barcodes section by name.
- Update validation logic to require barcodes
- test case for missing / missspecified barcodes section

**Step 2.2: Update Demultiplex implementation**
- Modify `init()` method to copy the barcodes.
- Maintain existing hamming correction logic temporarily

**Step 2.3: Update test cases for Demultiplex**
- Convert existing demultiplex test cases to use new [barcodes.<name>] sections
- Verify all existing functionality works
- Test both inline and referenced barcode configurations

### Phase 3: Implement HammingCorrect Step

**Step 3.1: Create HammingCorrect transformation**
- Create `src/transformations/hamming_correct.rs`
- Implement HammingCorrect struct with:
  - `label_in`: tag to correct
  - `label_out`: tag to store corrected result  
  - `barcodes_ref`: reference to barcodes section
  - `max_hamming_distance`: correction threshold
  - `on_no_match`: whether 
            - remove the tag
            - replace the tag's sequence with an empty sequence
            - do not change the tag.

**Step 3.2: Implement correction logic**
- Extract hamming correction algorithms from demultiplex.rs
- Implement both IUPAC and standard hamming distance correction
- Return corrected sequence or remove tag if no match within threshold depending on 'on_no_matc'h
- Preserve original tag if correction not needed

**Step 3.3: Add HammingCorrect to transformation enum**
- Update `Transformation` enum in transformations.rs
- Wire up parsing and execution

### Phase 4: Add Test Cases

**Step 4.1: Create unit tests**
- Test IUPAC hamming correction
- Test standard hamming correction  
- Test edge cases (no match, exact match, boundary distances)
- Test barcode section defined multiple times. 
- Test all on_no_match variants.
- test that label_in != label_out

**Step 4.2: Create integration test cases**
- Create test_cases/hamming_correct/ directory
- Add test cases with various barcode scenarios
- Test pipeline: Extract -> HammingCorrect -> Demultiplex

### Phase 5: Remove Hamming Logic from Demultiplex

**Step 5.1: Simplify Demultiplex**
- Remove `max_hamming_distance` field from Demultiplex struct
- Remove hamming correction logic from `apply()` method
- Simplify to only do exact matches against barcode_to_tag mapping

**Step 5.2: Update existing test cases**
- Convert test cases that use `max_hamming_distance` > 0  on Demultiplex
- Add HammingCorrect step before Demultiplex in these test cases
- Ensure same behavior is maintained

### Phase 6: Validation and Documentation

**Step 6.1: Run full test suite**
- Execute `dev/update_tests.py` to regenerate test outputs
- Run `cargo test` to verify all tests pass
- Check that no existing functionality is broken

**Step 6.2: Update template.toml**
- Add example HammingCorrect step configuration
- Update Demultiplex example to show barcodes reference
- Document the new [barcodes.<name>] section

**Step 6. Add documentation**
- add docs/content/docs/reference/Tag steps/HammingCorrect.md
- adjust Demultiplex documentation.

### Code Reuse Strategy
- Extract hamming distance functions to shared utility module if needed
- Share barcode resolution logic between HammingCorrect and Demultiplex
- Maintain existing DemultiplexInfo structure for backwards compatibility

### Migration Path  
- Phase 2 ensures existing configs continue working
- Users can gradually migrate to new [barcodes.<name>] + HammingCorrect approach
- Both patterns supported during transition period

### Error Handling
- Clear error messages when barcodes section is missing but referenced
- Validation errors for invalid barcode references
- Preserve existing error behavior for malformed barcode definitions

## Success Criteria
1. All existing demultiplex test cases pass with new implementation
2. HammingCorrect step works independently of demultiplex
3. No performance regression in processing pipeline
4. Configuration backwards compatibility maintained
5. New [barcodes.<name>] section enables code reuse between steps
