# AI Plan 31: Replace eserde with toml-pretty-deser

## Objective

Replace the current `eserde::Deserialize` deserialization system with `toml-pretty-deser` to gain:
- Pretty error messages with line/column highlighting
- Multiple errors reported at once
- Pin-point accuracy in error locations
- Case/snake-case insensitivity options

**Scope (Stage 1)**: This is a straight replacement - remove eserde/serde derive macros, replace with `#[tpd]` macros, convert aliases, tag nested structs, and replace deserialization calls. Validation logic migration is NOT part of this stage.

## Library Overview (from docs.rs/toml-pretty-deser)

**Note**: This plan assumes `toml-pretty-deser` version 0.2 which introduces the `#[tpd_skip]` attribute.

### Key Concepts

1. **`#[tpd_make_partial]`** - Applied to structs to generate a `PartialT` type
   - `#[tpd_make_partial(true)]` or `#[tpd_make_partial]` - auto-generates `VerifyFromToml` impl
   - `#[tpd_make_partial(false)]` - requires manual `VerifyFromToml` implementation

2. **`#[tpd_nested]`** - Tag fields that contain nested structs that also need deserialization

3. **`#[tpd_alias(name1, name2)]`** - Field aliases (replaces `#[serde(alias = "...")]`)

4. **`#[tpd_skip]`** - Skip field during deserialization (replaces `#[serde(skip)]`) - field won't be read from TOML and will use `Default::default()`

5. **`#[tpd_make_enum]`** - For simple string-typed enums without payloads

6. **`#[tpd_make_tagged_enum("tag_key", aliases = ["alias1"])]`** - For tagged enums with struct payloads

7. **`deserialize<PartialT, T>(toml_str)` / `deserialize_with_mode<PartialT, T>(...)`** - Entry points

### Field Types
- All struct fields get wrapped in `TomlValue<T>` in the Partial struct
- `#[tpd_skip]` fields are excluded from TOML parsing and use their default value
- `#[serde(default)]` becomes handled via the `TomlValue` system

---

## Step-by-Step Migration Guide

### Step 1: Add Dependency

**File**: `mbf-fastq-processor/Cargo.toml`

```toml
[dependencies]
toml-pretty-deser = "0.2"
```

Remove `eserde` dependency once migration is complete.

---

### Step 2: Create Prelude Module

**File**: `src/config/tpd_prelude.rs` (new file)

```rust
pub use toml_pretty_deser::prelude::*;
```

This provides the macros and types needed for deserialization.

---

### Step 3: Convert Struct Attributes

#### Pattern: Basic Struct

**Before** (eserde):
```rust
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MyStruct {
    pub field1: String,
    #[serde(default)]
    pub field2: Option<i32>,
    #[serde(default)]
    #[serde(skip)]
    pub computed_field: Option<ComputedType>,
}
```

**After** (toml-pretty-deser):
```rust
use crate::config::tpd_prelude::*;

#[tpd_make_partial]
#[derive(Debug, Clone, JsonSchema)]
pub struct MyStruct {
    pub field1: String,
    pub field2: Option<i32>,  // defaults work automatically via TomlValue
    #[tpd_skip]
    pub computed_field: Option<ComputedType>,  // not read from TOML, uses Default
}
```

**Note**: Fields marked `#[tpd_skip]` must implement `Default`. They will be initialized with their default value and not read from the TOML file.

---

#### Pattern: Aliases

**Before**:
```rust
#[serde(alias = "segment")]
pub source: String,
```

**After**:
```rust
#[tpd_alias(segment)]
pub source: String,
```

For multiple aliases:
```rust
// Before
#[serde(alias = "start")]
#[serde(alias = "Left")]
#[serde(alias = "left")]
Start,

// After
#[tpd_alias(start, Left, left)]
Start,
```

---

#### Pattern: Nested Structs

**Before**:
```rust
#[derive(eserde::Deserialize)]
pub struct Config {
    pub input: Input,
    pub output: Option<Output>,
}
```

**After**:
```rust
#[tpd_make_partial]
pub struct Config {
    #[tpd_nested]
    pub input: Input,
    #[tpd_nested]
    pub output: Option<Output>,
}
```

**Important**: Any struct type used as a field type must ALSO have `#[tpd_make_partial]` applied.

---

#### Pattern: Simple Enums (String-Typed)

**Before**:
```rust
#[derive(eserde::Deserialize, Debug, Clone)]
pub enum KeepOrRemove {
    #[serde(alias = "keep")]
    Keep,
    #[serde(alias = "remove")]
    Remove,
}
```

**After**:
```rust
#[tpd_make_enum]
#[derive(Debug, Clone)]
pub enum KeepOrRemove {
    #[tpd_alias(keep)]
    Keep,
    #[tpd_alias(remove)]
    Remove,
}
```

---

#### Pattern: Tagged Enums (With Struct Payloads)

The `Transformation` enum uses `#[serde(tag = "action")]`:

**Before**:
```rust
#[derive(eserde::Deserialize, Debug, strum_macros::Display, JsonSchema)]
#[serde(tag = "action")]
pub enum Transformation {
    CutStart(edits::CutStart),
    CutEnd(edits::CutEnd),
    #[serde(alias = "SpotCheckReadNames")]
    ValidateReadPairing(validation::ValidateReadPairing),
    #[serde(skip)]
    _ChangeCase(edits::_ChangeCase),
    // ...
}
```

**After**:
```rust
#[tpd_make_tagged_enum("action")]
#[derive(Debug, strum_macros::Display, JsonSchema)]
pub enum Transformation {
    CutStart(edits::CutStart),
    CutEnd(edits::CutEnd),
    #[tpd_alias(SpotCheckReadNames)]
    ValidateReadPairing(validation::ValidateReadPairing),
    #[tpd_skip]  // Internal-only variant, not deserializable from TOML
    _ChangeCase(edits::_ChangeCase),
    // ...
}
```

**Note**: `#[tpd_skip]` on enum variants marks them as internal-only - they cannot be deserialized from TOML. This is used for variants like `_ChangeCase`, `_ReportCount`, etc. that are created programmatically during config expansion.

---

### Step 4: Handle Custom Deserializers

The codebase has custom serde deserializers in `src/config/deser.rs`. These should be converted to `VerifyFromToml` implementations that call validation functions, not newtypes.

#### Pattern: Custom Validation via VerifyFromToml

**Example**: `u8_from_char_or_number` for a field that accepts either a single character or a number 0-255.

**Before** (serde):
```rust
#[serde(deserialize_with = "u8_from_char_or_number")]
pub comment_separator: u8,
```

**After** (tpd):

Use `#[tpd_make_partial(false)]` to write a manual `VerifyFromToml` implementation:

```rust
#[tpd_make_partial(false)]
#[derive(Debug, Clone)]
pub struct InputOptions {
    pub fasta_fake_quality: Option<u8>,
    pub read_comment_character: u8,
    // ... other fields
}

impl VerifyFromToml for PartialInputOptions {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self {
        // Validate read_comment_character using the existing validation logic
        self.read_comment_character = self.read_comment_character.verify(helper, |value| {
            validate_char_or_number(*value)
        });
        
        // Validate fasta_fake_quality range
        self.fasta_fake_quality = self.fasta_fake_quality.verify(helper, |value| {
            if let Some(v) = value {
                if *v < 33 || *v > 126 {
                    return Err((
                        format!("fasta_fake_quality must be in range [33..126], got {v}"),
                        None,
                    ));
                }
            }
            Ok(())
        });
        
        self
    }
}

// Keep existing validation functions, just adapt their signatures
fn validate_char_or_number(value: u8) -> Result<(), (String, Option<String>)> {
    // validation logic here - return Ok(()) or Err((message, hint))
    Ok(())
}
```

#### Custom Deserializers to Convert

| Serde Function | Approach |
|----------------|----------|
| `u8_from_char_or_number` | `VerifyFromToml` calling validation function |
| `opt_u8_from_char_or_number` | `VerifyFromToml` with `Option` handling |
| `bstring_from_string` | `BString` works directly with tpd |
| `dna_from_string` | `VerifyFromToml` calling `validate_dna()` |
| `iupac_from_string` | `VerifyFromToml` calling `validate_iupac()` |
| `btreemap_iupac_dna_string_from_string` | `VerifyFromToml` with `IndexMap` validation |
| `filename_or_filenames` | Handled by `VecMode::SingleToVec` - no custom code needed |
| `deserialize_map_of_string_or_seq_string` | Handled natively - see Step 7 |

**Note**: Use `IndexMap` instead of `BTreeMap` throughout - toml-pretty-deser uses IndexMap internally and it preserves insertion order.

---

### Step 5: Update Entry Points

**Files to modify**:
- `src/cli/process.rs`
- `src/cli/validate.rs`  
- `src/cli/verify.rs`

**Before**:
```rust
let parsed = eserde::toml::from_str::<Config>(&raw_config)
    .map_err(|e| cli::improve_error_messages(e.into(), &raw_config))?;
```

**After**:
```rust
use toml_pretty_deser::{deserialize_with_mode, DeserError, FieldMatchMode, VecMode};

let result = deserialize_with_mode::<PartialConfig, Config>(
    &raw_config,
    FieldMatchMode::CaseInsensitive,
    VecMode::SingleToVec,  // allows `field = "single"` instead of `field = ["single"]`
);
let parsed = match result {
    Ok(config) => config,
    Err(e) => {
        bail!("{}", e.pretty(&raw_config, "config.toml"));
    }
};
```

**Note**: Always use `FieldMatchMode::CaseInsensitive` and `VecMode::SingleToVec` for all entry points. The `DeserError` type has a `.pretty()` method that handles all error formatting automatically.

---

### Step 6: Handle `#[serde(skip)]` Fields

Many structs have computed/runtime fields marked with `#[serde(skip)]`. With toml-pretty-deser 0.2, these can be handled directly with `#[tpd_skip]`.

**Before** (eserde):
```rust
#[derive(eserde::Deserialize)]
pub struct MyStruct {
    pub field1: String,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndex>,
}
```

**After** (toml-pretty-deser):
```rust
#[tpd_make_partial]
pub struct MyStruct {
    pub field1: String,
    #[tpd_skip]
    pub segment_index: Option<SegmentIndex>,  // Uses Default::default()
}
```

**Requirements**: 
- The field type must implement `Default`
- For `Option<T>` fields, this naturally defaults to `None`
- For other types, ensure `Default` is implemented appropriately

---

### Step 7: Handle `#[serde(flatten)]`

The `Input` struct uses `#[serde(flatten)]`:

```rust
#[serde(flatten, deserialize_with = "deserialize_map_of_string_or_seq_string")]
segments: BTreeMap<String, Vec<String>>,
```

**toml-pretty-deser approach**: This is handled natively. Simply use `IndexMap<String, Vec<String>>` for the segments field. Combined with `VecMode::SingleToVec` at the entry point, this automatically accepts both single strings and arrays of strings as values.

**After**:
```rust
#[tpd_make_partial]
pub struct Input {
    // ... other fields ...
    pub segments: IndexMap<String, Vec<String>>,  // No special attribute needed
}
```

The `VecMode::SingleToVec` mode (set at the entry point) takes care of accepting single strings as one-element vectors, so both of these are valid:
```toml
R1 = "path/to/file.fastq"
R2 = ["path/to/file1.fastq", "path/to/file2.fastq"]
```

---

## File-by-File Migration Checklist

### Priority 1: Core Configuration
- [ ] `src/config.rs` - `Config`, `Benchmark`, `CheckedConfig`
- [ ] `src/config/input.rs` - `Input`, `InputOptions`, `CompressionFormat`, `FileFormat`
- [ ] `src/config/output.rs` - `Output`
- [ ] `src/config/options.rs` - `Options`, `FailureOptions`, `FailOutputError`
- [ ] `src/config/segments.rs` - `Segment`, `SegmentOrAll`, `SegmentSequenceOrName`

### Priority 2: Transformations Main
- [ ] `src/transformations.rs` - `Transformation` enum, `RegionDefinition`, `RegionAnchor`, `KeepOrRemove`

### Priority 3: Individual Transformation Structs
Each file in `src/transformations/` subdirectories:

**edits/**
- [ ] `cut_start.rs` - `CutStart`
- [ ] `cut_end.rs` - `CutEnd`
- [ ] `truncate.rs` - `Truncate`
- [ ] `prefix.rs` - `Prefix`
- [ ] `postfix.rs` - `Postfix`
- [ ] `convert_quality.rs` - `ConvertQuality`
- [ ] `reverse_complement.rs` - `ReverseComplement`
- [ ] `rename.rs` - `Rename`
- [ ] `swap.rs` - `Swap`
- [ ] `lowercase.rs` - `Lowercase`
- [ ] `uppercase.rs` - `Uppercase`
- [ ] `trim_at_tag.rs` - `TrimAtTag`
- [ ] `merge_reads.rs` - `MergeReads`

**filters/**
- [ ] `head.rs` - `Head`
- [ ] `skip.rs` - `Skip`
- [ ] `empty.rs` - `FilterEmpty`
- [ ] `sample.rs` - `Sample`
- [ ] `reservoir_sample.rs` - `ReservoirSample`
- [ ] `by_tag.rs` - `ByTag`
- [ ] `by_numeric_tag.rs` - `ByNumericTag`

**validation/**
- [ ] `spot_check_read_pairing.rs` - `ValidateReadPairing`
- [ ] `seq.rs` - `ValidateSeq`
- [ ] `quality.rs` - `ValidateQuality`
- [ ] `name.rs` - `ValidateName`
- [ ] `all_reads_same_length.rs` - `ValidateAllReadsSameLength`

**extract/**
- [ ] `iupac.rs` - `IUPAC`
- [ ] `iupac_with_indel.rs` - `IUPACWithIndel`
- [ ] `regex.rs` - `Regex`
- [ ] `region.rs` - `Region`
- [ ] `regions.rs` - `Regions`
- [ ] `regions_of_low_quality.rs` - `RegionsOfLowQuality`
- [ ] `longest_poly_x.rs` - `LongestPolyX`
- [ ] `poly_tail.rs` - `PolyTail`
- [ ] `iupac_suffix.rs` - `IUPACSuffix`
- [ ] `low_quality_start.rs` - `LowQualityStart`
- [ ] `low_quality_end.rs` - `LowQualityEnd`

**extract/tag/**
- [ ] `duplicates.rs` - `Duplicates`
- [ ] `other_file_by_name.rs` - `OtherFileByName`
- [ ] `other_file_by_sequence.rs` - `OtherFileBySequence`

**calc/**
- [ ] `length.rs` - `Length`
- [ ] `base_content.rs` - `BaseContent`
- [ ] `gc_content.rs` - `GCContent`
- [ ] `n_count.rs` - `NCount`
- [ ] `complexity.rs` - `Complexity`
- [ ] `qualified_bases.rs` - `QualifiedBases`
- [ ] `expected_error.rs` - `ExpectedError`
- [ ] `kmers.rs` - `Kmers`

**convert/**
- [ ] `regions_to_length.rs` - `RegionsToLength`
- [ ] `eval_expression.rs` - `EvalExpression`

**tag/**
- [ ] `store_tag_in_sequence.rs` - `StoreTagInSequence`
- [ ] `replace_tag_with_letter.rs` - `ReplaceTagWithLetter`
- [ ] `concat_tags.rs` - `ConcatTags`
- [ ] `forget_all_tags.rs` - `ForgetAllTags`
- [ ] `forget_tag.rs` - `ForgetTag`
- [ ] `store_tag_in_comment.rs` - `StoreTagInComment`
- [ ] `store_tag_in_fastq.rs` - `StoreTagInFastQ`
- [ ] `store_tag_location_in_comment.rs` - `StoreTagLocationInComment`
- [ ] `store_tags_in_table.rs` - `StoreTagsInTable`
- [ ] `quantify_tag.rs` - `QuantifyTag`

**reports/**
- [ ] `progress.rs` - `Progress`
- [ ] `report.rs` - `Report`
- [ ] `inspect.rs` - `Inspect`

**other/**
- [ ] `demultiplex.rs` - `Demultiplex`
- [ ] `hamming_correct.rs` - `HammingCorrect`

### Priority 4: Other Types
- [ ] `src/io/fileformats.rs` - `PhredEncoding`
- [ ] `src/dna.rs` - `IUPAC` enum

---

## Testing Strategy

1. **Keep existing tests running**: The test suite should pass after migration
2. **Test error messages**: Create test TOMLs with intentional errors to verify pretty output
3. **Test all aliases**: Ensure all serde aliases work as tpd_alias
4. **Test edge cases**:
   - Missing required fields
   - Unknown fields
   - Wrong types
   - Multiple errors in one file

**Important notes on test changes**:
- Some test cases using `expected_error.txt` or `expected_error_regex.txt` will fail after migration because error message formats will change. These need to be updated to match the new pretty-printed error format.
- May need to change the test runner approach from "change paths in TOML" to "symlink into test directory" to keep paths consistent for error messages. This ensures file paths in error output match expected patterns.

---

## Common Pitfalls

1. **Don't forget `#[tpd_nested]`** on struct fields that contain other tpd structs
2. **Every nested struct needs `#[tpd_make_partial]`**
3. **`#[tpd_skip]` fields must implement `Default`** - the field will be initialized with `Default::default()`
4. **Custom deserializers use `VerifyFromToml` implementations** calling validation functions, not newtypes or `FromTomlItem`
5. **The partial type is `Partial{StructName}`**, e.g., `PartialConfig`
6. **Internal-only enum variants use `#[tpd_skip]`** - they cannot be deserialized from TOML
7. **Always use `FieldMatchMode::CaseInsensitive` and `VecMode::SingleToVec`** at entry points for consistency

---

## Rollback Plan

There is no rollback. We're free to implement this.