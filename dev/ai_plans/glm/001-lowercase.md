# Unify LowercaseSequence and LowercaseTag into Single Lowercase Step

## Context

The codebase currently has two separate transformations for converting sequences to lowercase:
- `LowercaseSequence` (src/transformations/edits/lowercase_sequence.rs, 69 lines) - Converts entire read sequences to lowercase, supports segment selection and conditional execution via `if_tag`
- `LowercaseTag` (src/transformations/edits/lowercase_tag.rs, 46 lines) - Converts tag sequences to lowercase, operates on Location-type tags

Both transformations are nearly identical except for:
1. Target: Sequence segments vs stored tags
2. Parameters: `segment`/`if_tag` vs `in_label`

The goal is to unify these into a single `Lowercase` step that uses a `source` parameter resolved to a `ResolvedSource`, following the pattern used by other transformations like `ValidateAllReadsSameLength` and `Duplicates`.

## Numbered Implementation Plan

### 1. Create new unified Lowercase implementation
- Create `/project/mbf-fastq-processor/src/transformations/edits/lowercase.rs`
- Define new struct:
  ```rust
  #[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
  #[serde(deny_unknown_fields)]
  pub struct Lowercase {
      #[serde(default = "default_source")]
      #[serde(alias = "segment")]
      #[serde(alias = "source")]
      target: String,  // Gets resolved to ResolvedSource

      #[serde(default)]
      #[serde(skip)]
      resolved_target: Option<ResolvedSource>,

      #[serde(default)]
      if_tag: Option<String>,  // Conditional execution (from LowercaseSequence)
  }
  ```
- Implement `Step` trait:
  - `validate_segments()`: Parse source string to ResolvedSource using `ResolvedSource::parse()`
  - `uses_tags()`: Combine tag dependencies from both `resolved_source.get_tags()` and optional `if_tag`
  - `apply()`: Match on `resolved_source`:
    - `Segment(segment_index_or_all)`: Use existing pattern from LowercaseSequence with `apply_in_place_wrapped_plus_all()`
    - `Tag(tag_name)`: Use existing pattern from LowercaseTag to iterate through Location tags
    - `Name { segment, split_character }`: Convert read names to lowercase using `read.name_without_comment()` and `read.replace_name()`
  - Support `if_tag` conditional execution using existing `ConditionalTag` pattern

### 2. Update Transformation enum registration
- In `/project/mbf-fastq-processor/src/transformations.rs` (around lines 438-441):
  - Replace `LowercaseTag(edits::LowercaseTag)` and `LowercaseSequence(edits::LowercaseSequence)` with single `Lowercase(edits::Lowercase)` variant
  - Remove old enum variants entirely (no backward compatibility needed)

### 3. Delete old implementation files
- Delete `lowercase_sequence.rs`
- Delete `lowercase_tag.rs`
- Remove old exports from edits module if needed

### 4. Update documentation

#### A. Create new Lowercase documentation
- Create `/project/docs/content/docs/reference/modification-steps/Lowercase.md`:
  ```toml
  ---
  weight: 150
  ---

  # Lowercase

  Convert sequences, tags, or read names to lowercase.

  ```toml
  [[step]]
      action = "Lowercase"
      source = "read1"  # Any input segment, 'All', 'tag:mytag', or 'name:read1'
      if_tag = "mytag"  # Optional: only apply if tag is truthy
  ```

  ## Source Options

  - **Segment**: `"read1"`, `"read2"`, `"index1"`, `"index2"`, or `"All"` - lowercase the sequence
  - **Tag**: `"tag:mytag"` - lowercase the tag's sequence content (Location-type tags only)
  - **Name**: `"name:read1"` - lowercase the read name (not including comments)

  Optionally only applies if a [tag]({{< relref "docs/concepts/tag.md" >}}) is truthy via `if_tag`.

  ## Examples

  ### Lowercase a segment
  ```toml
  [[step]]
      action = "Lowercase"
      target = "read1"
  ```

  ### Lowercase a tag
  ```toml
  [[step]]
      action = "ExtractIUPAC"
      search = "CTN"
      out_label = "mytag"

  [[step]]
      action = "Lowercase"
      target = "tag:mytag"
  ```

  Follow with [StoreTagInSequence]({{< relref "docs/reference/tag-steps/using/StoreTagInSequence.md" >}}) to apply the lowercase tag back onto the read.

  ### Lowercase read names
  ```toml
  [[step]]
      action = "Lowercase"
      target = "name:read1"
  ```

  ### Conditional lowercase
  ```toml
  [[step]]
      action = "Lowercase"
      target = "read1"
      if_tag = "is_valid"
  ```
  ```

#### B. Delete old documentation
- Delete `/project/docs/content/docs/reference/modification-steps/LowercaseSequence.md`
- Delete `/project/docs/content/docs/reference/modification-steps/LowercaseTag.md`

#### C. Update template.toml
- In `/project/mbf-fastq-processor/src/template.toml` (around lines 580-609):
  - Replace the old entries with:
    ```toml
    # ==== Lowercase ====
    ## Convert sequences, tags, or read names to lowercase
    # [[step]]
    #    action = "Lowercase"
    #    source = "read1"  # Any input segment, 'All', 'tag:mytag', or 'name:segment'
    #    #if_tag = "mytag"  # (optional) Only lowercase where this tag is true
    #
    ## For tags: use source = "tag:mytag" to lowercase tag content
    ## You still want to StoreTagInSequence after this to actually change sequence
    ## For names: use source = "name:read1" to lowercase read names

    # ==== Uppercase ====
    ## Convert sequences, tags, or read names to uppercase
    # [[step]]
    #    action = "Uppercase"
    #    source = "read1"  # Any input segment, 'All', 'tag:mytag', or 'name:segment'
    #    #if_tag = "mytag"  # (optional) Only uppercase where this tag is true
    #
    ## For tags: use source = "tag:mytag" to uppercase tag content
    ## For names: use source = "name:read1" to uppercase read names
    ```

### 5. Update test cases

#### Primary test cases to migrate immediately:

1. `/project/test_cases/single_step/edits/lowercase_sequence/input.toml`
   - Change: `action = 'LowercaseSequence'` → `action = 'Lowercase'`
   - No config changes needed (default `source = "read1"` matches old behavior)

2. `/project/test_cases/single_step/edits/lowercase_tag/input.toml`
   - Change: `action = 'LowercaseTag'` → `action = 'Lowercase'`
   - Change: `in_label = 'test'` → `source = 'tag:test'`

#### Secondary test cases (19 files using LowercaseTag in extraction workflows):

All of these use `LowercaseTag` after extraction steps. Change each `action = 'LowercaseTag'` and `in_label = 'X'` to `action = 'Lowercase'` and `source = 'tag:X'`:

3. `/project/test_cases/single_step/extraction/extract_region/and_replace_multiple/input.toml`
4. `/project/test_cases/single_step/extraction/extract_region/read_too_short/input.toml`
5. `/project/test_cases/single_step/extraction/overlapping_regions_trim_conflict/input.toml`
6. `/project/test_cases/single_step/extraction/extract_iupac/multiple/input.toml`
7. `/project/test_cases/single_step/extraction/extract_highlight/basic/input.toml`
8. `/project/test_cases/single_step/extraction/extract_highlight/regex/input.toml`
9. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/trim_quality_start/input.toml`
10. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/swap/input.toml` (2 instances)
11. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/swap_conditional/input.toml` (2 instances)
12. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/rev_complement/input.toml`
13. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/prefix/input.toml`
14. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/max_len_inside_tag/input.toml`
15. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/max_len_before_tag/input.toml`
16. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/max_len_after_tag/input.toml`
17. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_true/input.toml` (2 instances)
18. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_false/input.toml` (2 instances)
19. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_true/input.toml` (2 instances)
20. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_false/input.toml` (2 instances)
21. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/cut_start_inside_tag/input.toml`
22. `/project/test_cases/single_step/extraction/edits_altering_tag_locations/cut_end_inside_tag/input.toml`

#### Error handling test:

23. `/project/test_cases/single_step/error_handling/malformed_fastq/quality_starts_with_at/input.toml`
   - Change: `action = 'LowercaseSequence'` → `action = 'Lowercase'`

### 6. Regenerate test harness
- Run `python3 dev/update_tests.py` to regenerate `/project/mbf-fastq-processor/tests/generated.rs` with updated test functions
- Verify all 23 test cases are properly generated

### 7. Run full test suite
- Run `cargo test` to verify all tests pass with new implementation
- Run `cargo test -- --ignored` to cover any long-running tests
- Run `cargo clippy --all-targets -- -D clippy::pedantic` to ensure code quality

### 8. Add test coverage for Name variant
- Create new test case `/project/test_cases/single_step/edits/lowercase_name/` to verify lowercase on read names
- Input FASTQ with mixed-case read names
- Expected output with lowercased read names
- Run `python3 dev/update_tests.py` to include new test

## Key Implementation Details

### ResolvedSource handling
The unified implementation will leverage the existing `ResolvedSource` enum:

```rust
match resolved_source {
    ResolvedSource::Segment(segment_index_or_all) => {
        // Use apply_in_place_wrapped_plus_all like LowercaseSequence
        block.apply_in_place_wrapped_plus_all(
            segment_index_or_all,
            |read| {
                let seq = read.seq().to_vec();
                let new_seq: Vec<u8> = seq.iter().map(|&b| b.to_ascii_lowercase()).collect();
                read.replace_seq_keep_qual(&new_seq);
            },
            condition.as_deref(),
        );
    }
    ResolvedSource::Tag(tag_name) => {
        // Iterate through Location tags like LowercaseTag
        let hits = block.tags.get_mut(tag_name).expect("Tag missing");
        for tag_val in hits.iter_mut() {
            if let TagValue::Location(hit) = tag_val {
                for hit_region in &mut hit.0 {
                    for ii in 0..hit_region.sequence.len() {
                        hit_region.sequence[ii] = hit_region.sequence[ii].to_ascii_lowercase();
                    }
                }
            }
        }
    }
    ResolvedSource::Name { segment, split_character } => {
        // NEW: Lowercase read names
        let mut pseudo_iter = block.get_pseudo_iter();
        while let Some(read) = pseudo_iter.pseudo_next() {
            let name = read.segments[segment].name_without_comment(*split_character);
            let lowercased: Vec<u8> = name.iter().map(|&b| b.to_ascii_lowercase()).collect();
            read.segments[segment].replace_name(&lowercased, *split_character);
        }
    }
}
```

## Test Coverage Strategy

1. **Integration tests**: All 24 existing test cases provide coverage for real-world usage patterns
   - 2 primary test cases for basic functionality
   - 19 secondary test cases for tag workflows
   - 1 error handling test for malformed input
   - 1 NEW test case for name lowercase

2. **Regression tests**: The existing error handling test ensures the step doesn't crash on malformed input

## Benefits of Unification

1. **Reduced code duplication**: From 115 lines to ~80 lines
2. **Consistent API**: Same `source` parameter as other transformations
3. **Easier maintenance**: Single implementation to maintain and improve
4. **Better discoverability**: Users find one Lowercase step instead of two
5. **Future-proof**: Easily extendable to support Name sources or other variants
6. **Enhanced functionality**: Support for lowercase on read names (not previously available)
