# Proposal: Conditional Read-Editing Steps

## Overview
Make all read-editing transformations optionally conditional based on boolean tags with minimal implementation work.

## Recommended Approach: Optional `if_tag` Field

Add an optional `if_tag: Option<String>` field to each editing step struct. When present, the transformation only applies to reads where the tag has a truthy value.

## Implementation Pattern

### 1. Struct Modification (per step)
```rust
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CutStart {
    n: usize,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    // NEW: Optional conditional tag
    #[serde(default)]
    if_tag: Option<String>,
}
```

### 2. Add `uses_tags()` Implementation
```rust
impl Step for CutStart {
    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        self.if_tag.as_ref().map(|tag| {
            vec![(tag.clone(), &[TagValueType::Bool])]
        })
    }

    // ... rest of implementation
}
```

### 3. Modify `apply()` Method

Two patterns depending on the current implementation:

#### Pattern A: For steps using `apply_in_place`
Replace `apply_in_place` with conditional iteration:

```rust
fn apply(
    &mut self,
    mut block: FastQBlocksCombined,
    _input_info: &InputInfo,
    _block_no: usize,
    _demultiplex_info: &OptDemultiplex,
) -> anyhow::Result<(FastQBlocksCombined, bool)> {
    let segment_idx = self.segment_index.unwrap().get_index();

    if let Some(ref tag_label) = self.if_tag {
        // Conditional mode: only apply to reads where tag is truthy
        let tag_values: Vec<bool> = block
            .tags
            .get(tag_label)
            .expect("Tag validation should have caught this")
            .iter()
            .map(|tv| tv.truthy_val())
            .collect();

        for (idx, should_apply) in tag_values.iter().enumerate() {
            if *should_apply {
                block.segments[segment_idx].entries[idx].cut_start(self.n);
            }
        }

        // Update tag locations conditionally
        filter_tag_locations(
            &mut block,
            self.segment_index.unwrap(),
            |location, pos, _seq, _read_len| {
                if tag_values[pos] {
                    if location.start < self.n {
                        NewLocation::Remove
                    } else {
                        NewLocation::New(HitRegion {
                            start: location.start - self.n,
                            len: location.len,
                            segment_index: location.segment_index,
                        })
                    }
                } else {
                    NewLocation::Keep
                }
            },
        );
    } else {
        // Original unconditional mode
        apply_in_place(
            self.segment_index.unwrap(),
            |read| read.cut_start(self.n),
            &mut block,
        );

        filter_tag_locations(
            &mut block,
            self.segment_index.unwrap(),
            |location, _pos, _seq, _read_len| {
                if location.start < self.n {
                    NewLocation::Remove
                } else {
                    NewLocation::New(HitRegion {
                        start: location.start - self.n,
                        len: location.len,
                        segment_index: location.segment_index,
                    })
                }
            },
        );
    }

    Ok((block, true))
}
```

## Scope: Steps to Modify

### Core Edit Steps (13 steps)
1. ✅ **CutStart** - Cut bases from start
2. ✅ **CutEnd** - Cut bases from end
3. ✅ **Truncate** - Limit to max length
4. ✅ **Prefix** - Add sequence to start
5. ✅ **Postfix** - Add sequence to end
6. ✅ **ConvertQuality** - Convert quality encoding
7. ✅ **ReverseComplement** - Reverse complement (already has `ReverseComplementConditional`)
8. ✅ **Rename** - Change read names
9. ✅ **Swap** - Swap segments (already has `SwapConditional`)
10. ✅ **LowercaseTag** - Convert tag to lowercase
11. ✅ **UppercaseTag** - Convert tag to uppercase
12. ✅ **LowercaseSequence** - Convert sequence to lowercase
13. ✅ **UppercaseSequence** - Convert sequence to uppercase

### Special Cases
14. **TrimAtTag** - Already tag-based, probably doesn't need `if_tag`
15. **MergeReads** - Complex, may need special handling

## Alternative: Helper Function Pattern

To reduce code duplication, create a helper function:

```rust
// In transformations/edits/mod.rs or similar
pub(crate) fn apply_conditional<F, G>(
    block: &mut FastQBlocksCombined,
    segment_index: SegmentIndex,
    if_tag: &Option<String>,
    unconditional_fn: F,
    conditional_fn: G,
) where
    F: Fn(&mut io::FastQRead),
    G: Fn(&mut io::FastQRead, bool),
{
    if let Some(ref tag_label) = if_tag {
        let tag_values: Vec<bool> = block
            .tags
            .get(tag_label)
            .expect("Tag validation should have caught this")
            .iter()
            .map(|tv| tv.truthy_val())
            .collect();

        let segment_idx = segment_index.get_index();
        for (idx, should_apply) in tag_values.iter().enumerate() {
            conditional_fn(&mut block.segments[segment_idx].entries[idx], *should_apply);
        }
    } else {
        apply_in_place(segment_index, unconditional_fn, block);
    }
}
```

Then each step becomes simpler:
```rust
apply_conditional(
    &mut block,
    self.segment_index.unwrap(),
    &self.if_tag,
    |read| read.cut_start(self.n),
    |read, should_apply| {
        if should_apply {
            read.cut_start(self.n)
        }
    },
);
```

## Effort Estimate

- **Helper function:** ~50 lines
- **Per-step modification:** ~15-30 lines each
- **Tests per step:** ~20 lines
- **Total:** ~500-800 lines for all steps

## Benefits

1. **Consistent API** - Same pattern across all steps
2. **Backward compatible** - `if_tag` is optional
3. **Type-safe** - Validated at config parse time
4. **Performant** - No overhead when not used
5. **Clear intent** - Explicit in TOML config

## Example Usage

```toml
# Extract a tag marking reads that need trimming
[[step]]
action = "ExtractRegex"
out_label = "has_adapter"
pattern = "AGATCGGAAGAG"
segment = "read1"

# Convert to boolean (found/not found)
[[step]]
action = "EvalExpression"
out_label = "needs_trim"
expression = "has_adapter != ''"

# Conditionally trim only reads with adapters
[[step]]
action = "CutStart"
n = 10
segment = "read1"
if_tag = "needs_trim"
```

## Migration Path for Existing Conditional Steps

Two options:

### Option A: Keep both variants
- Keep `ReverseComplementConditional` and `SwapConditional` as-is
- Add `if_tag` to base `ReverseComplement` and `Swap`
- Deprecate conditional variants in docs
- Remove conditional variants in next major version

### Option B: Immediate consolidation
- Modify base steps to support `if_tag`
- Update expansion logic to convert old conditional variants to new syntax
- Maintain backward compatibility during TOML parsing

Recommendation: **Option A** for smoother transition.
