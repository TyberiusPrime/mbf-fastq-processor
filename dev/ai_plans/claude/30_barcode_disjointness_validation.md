# AI Plan 30: Barcode Disjointness Validation

## Problem Statement

Currently, the barcode demultiplexing system only ensures that barcodes are lexicographically distinct but doesn't validate that IUPAC barcode patterns are truly disjoint. This means multiple barcodes could potentially accept the same sequence when IUPAC ambiguity codes are involved, leading to non-deterministic classification outcomes.

## Current Implementation Analysis

### Barcode Matching Logic (`src/demultiplex.rs:52-64`)
The current `barcode_to_tag` function:
1. First tries exact string matching 
2. Falls back to IUPAC matching using `iupac_hamming_distance`
3. Returns the **first** match found in iteration order

### IUPAC Matching (`src/dna.rs`)
The `iupac_hamming_distance` function supports:
- Standard IUPAC ambiguity codes (R, Y, S, W, K, M, B, D, H, V, N)
- Case-insensitive matching
- Returns 0 for perfect matches (including ambiguous matches)

### Current Validation (`src/demultiplex.rs:26-28`)
Only validates that output names aren't "no-barcode" - no overlap validation.

## Test Case Design

### Test Case: `overlapping_iupac_barcodes` (Expected to Fail)

**Objective**: Create a test case with overlapping IUPAC barcodes that should fail validation.

**Template**: Based on `test_cases/input_validation/barcode_outputs_not_named_no_barcode/`

**Structure**:
```
test_cases/input_validation/overlapping_iupac_barcodes/
├── input.toml
└── expected_panic.txt
```

**Test Barcodes** (deliberately overlapping):
- `"NNNN"` → matches any 4-base sequence  
- `"ATCG"` → matches exactly ATCG
- `"RYRN"` → matches A/G-C/T-A/G-any (includes ATCG)

**Expected Behavior**: Sequence "ATCG" could match any of these three barcodes, making classification non-deterministic.

**TOML Configuration**:
```toml
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'

[[step]]
    action = 'ExtractRegion'
    start = 0
    length = 4
    label = 'barcode'

[[step]]
    action = 'Demultiplex'
    label = 'barcode'
    barcodes = 'test_barcodes'
    output_unmatched = false

[barcodes.test_barcodes]
    NNNN = 'universal'
    ATCG = 'specific'
    RYRN = 'ambiguous'
```

## Solution Strategy

### Phase 1: Overlap Detection Algorithm

**Core Algorithm**: For each pair of barcodes, enumerate all possible sequences they could match and check for intersections.

**Implementation Approach**:
```rust
fn detect_barcode_overlaps(barcodes: &BTreeMap<BString, String>) -> Result<()> {
    let barcode_patterns: Vec<_> = barcodes.keys().collect();
    
    for i in 0..barcode_patterns.len() {
        for j in (i+1)..barcode_patterns.len() {
            if barcodes_overlap(barcode_patterns[i], barcode_patterns[j])? {
                bail!("Barcodes '{}' and '{}' have overlapping accepted sequences", 
                      String::from_utf8_lossy(barcode_patterns[i]),
                      String::from_utf8_lossy(barcode_patterns[j]));
            }
        }
    }
    Ok(())
}
```

### Phase 2: Overlap Checking Implementation

**Option A: Enumeration Approach**
- Generate all possible sequences for each IUPAC pattern
- Check for set intersection
- **Pros**: Conceptually simple, exhaustive
- **Cons**: Exponential complexity (4^n sequences for n positions)

**Option B: Constraint Satisfaction**
- Model each barcode as constraints
- Use constraint solving to find overlapping solutions
- **Pros**: More efficient for sparse patterns
- **Cons**: More complex implementation

**Option C: Direct Pattern Analysis**
- Analyze IUPAC patterns position-by-position
- Two patterns overlap if every position has compatible symbols
- **Pros**: Linear complexity, elegant
- **Cons**: Requires careful IUPAC logic

### Phase 3: Recommended Implementation (Option C)

```rust
fn barcodes_overlap(pattern1: &[u8], pattern2: &[u8]) -> Result<bool> {
    if pattern1.len() != pattern2.len() {
        return Ok(false);
    }
    
    for (c1, c2) in pattern1.iter().zip(pattern2.iter()) {
        if !positions_compatible(*c1, *c2) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn positions_compatible(c1: u8, c2: u8) -> bool {
    let set1 = iupac_to_bases(c1);
    let set2 = iupac_to_bases(c2);
    !set1.is_disjoint(&set2)
}
```

### Phase 4: Integration Points

**Location**: Add validation to `check_barcodes` in `src/config/mod.rs`

**Integration**:
```rust
    fn check_barcodes(&self, errors: &mut Vec<anyhow::Error>) {
        // Existing validation...
        
        // NEW: Validate barcode disjointness
        validate_barcode_disjointness(barcodes)?;
        
        // Rest of existing logic...
    }
}
```

## Success Criteria

1. **Test Case Creation**: New failing test case demonstrates the overlap problem
2. **Validation Implementation**: Robust overlap detection catches all IUPAC conflicts
3. **Performance**: Validation completes in reasonable time for typical barcode sets (< 100 barcodes)
4. **Backward Compatibility**: Existing valid configurations continue to work
5. **Error Messages**: Clear, actionable error messages for overlapping barcodes

## Edge Cases to Consider

- **Case sensitivity**: "ATCG" vs "atcg", already handled by deserialisation.
- **Empty barcodes**: must be rejected (add test case?)
- **Barcodes of differing lengths": reject for now (add test case)
- **Maximum barcode count**: Performance with large barcode sets
- **Complex IUPAC patterns**: Nested ambiguity codes like "NNRYNN"

## Implementation Priority

1. **High**: Create failing test case
2. **High**: Implement Option C (direct pattern analysis)
3. **Medium**: Integration into validation pipeline
4. **Medium**: Comprehensive test coverage
5. **Low**: Performance optimization for large barcode sets

This plan ensures barcode classification determinism while maintaining the flexibility of IUPAC pattern matching.
