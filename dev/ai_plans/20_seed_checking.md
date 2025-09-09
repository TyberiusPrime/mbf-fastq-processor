## Detailed Implementation Plan: Make Seeds Optional When false_positive_rate = 0.0

### Overview
Currently, three transformation steps require both a `seed` and `false_positive_rate` field, but the seed is only used when `false_positive_rate > 0.0` (for approximate filtering). When `false_positive_rate = 0.0`, exact filtering is used (HashSet) and the seed is ignored. This plan makes the seed optional and validates that it's provided only when needed.

### Affected Steps
1. **OtherFileBySequence** (`src/transformations/extract/tag/other_file_by_sequence.rs`)
2. **OtherFileByName** (`src/transformations/extract/tag/other_file_by_name.rs`) 
3. **Duplicates** (`src/transformations/extract/tag/duplicates.rs`)

Note: The `Sample` step in `src/transformations/filters/sample.rs` always uses its seed for randomization, so it's unaffected.

### Implementation Steps

#### 1. Update Struct Definitions
Change `seed: u64` to `seed: Option<u64>` in all three structs:

**File: `src/transformations/extract/tag/other_file_by_sequence.rs:22`**
```rust
// Change from:
pub seed: u64,
// To:
pub seed: Option<u64>,
```

**File: `src/transformations/extract/tag/other_file_by_name.rs:25`**
```rust
// Change from:
pub seed: u64,
// To:
pub seed: Option<u64>,
```

**File: `src/transformations/extract/tag/duplicates.rs:71`**
```rust
// Change from:
pub seed: u64,
// To:
pub seed: Option<u64>,
```

#### 2. Add Validation Logic
Implement custom validation in each struct's `validate()` method to ensure:
- If `false_positive_rate > 0.0`, then `seed` must be `Some(value)`
- If `false_positive_rate == 0.0`, then `seed` should be `None` (warn if provided)

**For OtherFileBySequence and OtherFileByName:**
Add validation after line 42 in their respective `validate()` methods:

```rust
// Validate seed requirement based on false_positive_rate
if self.false_positive_rate > 0.0 {
    if self.seed.is_none() {
        return Err(anyhow::anyhow!(
            "seed is required when false_positive_rate > 0.0 (approximate filtering)"
        ));
    }
} else if self.seed.is_some() {
    eprintln!(
        "Warning: seed provided but will be ignored when false_positive_rate = 0.0 (exact filtering)"
    );
}
```

**For Duplicates:**
Add validation after line 83 in the `validate()` method:

```rust
// Validate seed requirement based on false_positive_rate
if self.false_positive_rate > 0.0 {
    if self.seed.is_none() {
        return Err(anyhow::anyhow!(
            "seed is required when false_positive_rate > 0.0 (approximate filtering)"
        ));
    }
} else if self.seed.is_some() {
    eprintln!(
        "Warning: seed provided but will be ignored when false_positive_rate = 0.0 (exact filtering)"
    );
}
```

#### 3. Update Filter Initialization Logic
Modify the `init()` methods to handle optional seeds:

**File: `src/transformations/extract/tag/other_file_by_sequence.rs:64-72`**
```rust
let mut filter: ApproxOrExactFilter = if self.false_positive_rate == 0.0 {
    ApproxOrExactFilter::Exact(HashSet::new())
} else {
    let seed = self.seed.expect("seed should be validated to exist when false_positive_rate > 0.0");
    ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
        seed,
        100_000,
        self.false_positive_rate,
    )))
};
```

**File: `src/transformations/extract/tag/other_file_by_name.rs:70-78`**
```rust
let mut filter: ApproxOrExactFilter = if self.false_positive_rate == 0.0 {
    ApproxOrExactFilter::Exact(HashSet::new())
} else {
    let seed = self.seed.expect("seed should be validated to exist when false_positive_rate > 0.0");
    ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
        seed,
        100_000,
        self.false_positive_rate,
    )))
};
```

**File: `src/transformations/extract/tag/duplicates.rs:97-105`**
```rust
let filter: ApproxOrExactFilter = if self.false_positive_rate == 0.0 {
    ApproxOrExactFilter::Exact(HashSet::new())
} else {
    let seed = self.seed.expect("seed should be validated to exist when false_positive_rate > 0.0");
    ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
        seed,
        1_000_000,
        self.false_positive_rate,
    )))
};
```

#### 4. Update Test Cases and Documentation
After implementation:

1. **Update existing test cases** that use these steps with `false_positive_rate = 0.0`:
   - Remove `seed` field from TOML configs where `false_positive_rate = 0.0`
   - Add test cases that verify validation works (seed required when FPR > 0, optional when FPR = 0)

2. **Update template and documentation**:
   - Update `src/template.toml` to show seed as optional
   - Update relevant markdown files in `docs/` directory

#### 5. Testing Strategy
1. Run existing tests to ensure no regressions: `cargo test`
2. Add new test cases covering:
   - Validation error when FPR > 0 and no seed provided
   - Warning when FPR = 0 and seed provided
   - Successful operation in both exact and approximate modes
3. Run `dev/update_tests.py` followed by `cargo test` after adding test cases

#### 6. Error Handling
- Use `expect()` with descriptive messages when unwrapping optional seeds
- Provide clear validation error messages
- Use `eprintln!` for warnings about unused seeds

### Benefits
1. **Clearer Intent**: Configuration files clearly show when randomization is used
2. **Validation**: Prevents configuration errors (missing seeds when needed)
3. **Backward Compatibility**: Existing configs with exact filtering will show warnings but continue working
4. **Performance**: No change to runtime performance, only configuration clarity

### Files to Modify
1. `src/transformations/extract/tag/other_file_by_sequence.rs` - Lines 22, ~42, ~68
2. `src/transformations/extract/tag/other_file_by_name.rs` - Lines 25, ~42, ~74  
3. `src/transformations/extract/tag/duplicates.rs` - Lines 71, ~83, ~100
4. Test cases using these steps with `false_positive_rate = 0.0`
5. `src/template.toml` and relevant documentation files