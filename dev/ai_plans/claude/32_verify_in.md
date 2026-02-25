# AI Plan 32: Migrate remaining `check_*` functions to `VerifyIn` traits

## Objective

Move the remaining post-deserialization validation in `Config::check_*` methods (which collect errors into `Vec<anyhow::Error>`) into the `VerifyIn` trait system on `Partial*` types. This gives us:
- Precise source-span error highlighting (line/column in TOML)
- Multiple errors reported simultaneously with context
- Consistent error presentation (same format as all other validation)

## Current State

### Already migrated to `VerifyIn` (no changes needed)
- `PartialConfig::verify` — defaults, report cross-check, barcode validation
- `PartialInput::verify` — segment structure, stdin magic, segment name validation
- `PartialInputOptions::verify` — fasta_fake_quality range, rapidgzip/index conflict
- `PartialOutput::verify` — ix_separator, chunksize, compression_level, stdout, segment cross-check
- `PartialOptions::verify` — defaults, block_size even for interleaved
- `PartialBarcodes::verify` — IUPAC key validation
- All individual transformation `VerifyIn` impls (segment validation, field-level checks)

### Remaining `check_*` functions on `Config` (in `inner_check`)
These run AFTER deserialization succeeds and operate on concrete `Config`/`Transformation` types. They use `anyhow::Error` strings with no source spans.

| # | Function | Difficulty | Notes |
|---|----------|-----------|-------|
| 1 | `check_name_collisions` | Medium | Cross-cutting: compares segment names, barcode names, and tag labels |
| 2 | `check_transformations` | Hard | The big one: tag flow analysis (declare/use/remove), type checking, unused tag warnings. Calls `validate_others` on each Step. |
| 3 | `check_for_any_output` | Easy | Checks at least one form of output exists |
| 4 | `check_input_format` | Hard | Opens files to detect format; validates format consistency per segment, checks BAM/FASTA options |
| 5 | `check_input_format_for_validation` | Easy | Subset of above for validation mode |
| 6 | `check_input_duplicate_files` | Medium | Checks for repeated filenames across segments |
| 7 | `check_blocksize` | Easy | Already partly in PartialOptions::verify; remainder checks interleaved |
| 8 | `check_head_rapidgzip_conflict` | Easy | Cross-cutting: Head transform vs build_rapidgzip_index |
| 9 | `check_benchmark` | Easy | molecule_count > 0 |

### `validate_others` on `Step` trait (called from `check_transformations`)
These are post-concrete validation methods that need access to the full transform list. Each `bail!()` becomes a generic `anyhow::Error` with a `[Step N (name)]:` prefix. Implementations:

| Transform | What it validates |
|-----------|------------------|
| `Report` | Duplicate report names; count_oligos DNA validation |
| `Demultiplex` | Whether `output_unmatched` is required based on upstream tag type |
| `Progress` | stdout + progress conflict |
| `StoreTagsInTable` | At least one tag declared upstream |
| `StoreTagInComment` | Output segment exists in output config |
| `ConcatTags` | >= 2 input labels, no duplicates, separator not empty |
| `Prefix` | seq.len() == qual.len() |
| `Postfix` | seq.len() == qual.len() |
| `Regex` | Footgun detection for `$1_` group references |
| `TrimAtTag` | ExtractRegions single-entry constraint |
| `MergeReads` | min_overlap, max_mismatch_rate, spacer_quality_char ranges |
| `PolyTail` | min_length >= 2, max_mismatch_rate range |
| `LongestPolyX` | min_length > 0, max_mismatch_rate range |
| `RegionsToLength` | out_label != in_label |
| `Duplicates` | seed/false_positive_rate validation |
| `OtherFileBySequence` | seed/false_positive_rate validation |
| `OtherFileByName` | StoreTagInComment char compatibility |
| `HammingCorrect` | No-op (returns Ok) |

---

## Migration Strategy

### Guiding Principles

1. **Easy wins first** — start with simple, self-contained checks that only look at one section
2. **`validate_others` moves into `VerifyIn`** — most `validate_others` implementations only check the transform's own fields and can move into the existing `PartialX::verify()`. A few need cross-transform context (see below).
3. **Cross-cutting checks stay in `check_*` for now if they inherently need the full concrete config** — some checks (like tag flow analysis) genuinely need the ordered list of concrete transforms. These can remain but should be converted to produce span-aware errors where possible.
4. **File I/O checks (`check_input_format`) stay post-deserialization** — they open real files, which is inherently a runtime concern. But the error messages can be improved to reference spans.

### Phase 1: Move simple `validate_others` into `VerifyIn` (Easy)

These `validate_others` methods only check the transform's own fields and don't need `_all_transforms` or `_input_def`/`_output_def`. They should become part of the existing `VerifyIn<PartialConfig>` impl for each `PartialX` type.

**Pattern**: In each transform's `VerifyIn::verify()`, add field-level `.verify()` calls:

```rust
// Example: PartialPolyTail (currently in validate_others)
impl VerifyIn<PartialConfig> for PartialPolyTail {
    fn verify(&mut self, parent: &PartialConfig) -> Result<(), ValidationFailure> {
        self.segment.validate_segment(parent);
        self.min_length.verify(|v| {
            if *v < 2 {
                Err(ValidationFailure::new(
                    "min_length must be >= 2",
                    Some("Change to a positive integer larger than 1"),
                ))
            } else { Ok(()) }
        });
        self.max_mismatch_rate.verify(|v| {
            if *v < 0.0 || *v >= 1.0 {
                Err(ValidationFailure::new(
                    "max_mismatch_rate must be in [0.0..1.0)",
                    Some("Set a valid value >= 0 and < 1.0"),
                ))
            } else { Ok(()) }
        });
        Ok(())
    }
}
```

**Transforms to migrate in Phase 1** (simple field validation only):

| Transform file | Fields to validate | Current location |
|---|---|---|
| `extract/poly_tail.rs` | `min_length >= 2`, `max_mismatch_rate` range | `validate_others` |
| `extract/longest_poly_x.rs` | `min_length > 0`, `max_mismatch_rate` range | `validate_others` |
| `edits/merge_reads.rs` | `min_overlap >= 5`, `max_mismatch_rate` range, `spacer_quality_char` range | `validate_others` |
| `edits/prefix.rs` | `seq.len() == qual.len()` | `validate_others` |
| `edits/postfix.rs` | `seq.len() == qual.len()` | `validate_others` |
| `extract/regex.rs` | `$1_` footgun in replacement | `validate_others` |
| `convert/regions_to_length.rs` | `out_label != in_label` | `validate_others` |
| `hamming_correct.rs` | No-op — remove the `validate_others` override entirely | `validate_others` |

**For each transform**:
1. Move the validation logic from `validate_others` into `VerifyIn::verify()` using `.verify(|v| ...)` on the relevant `TomlValue` field
2. For cross-field checks (e.g., `seq.len() == qual.len()` in Prefix), use `TomlValueState::Custom { spans }` on both fields to show both locations
3. Delete the now-empty `validate_others` override (or leave the default no-op)
4. After migration: `cargo fmt`, then run tests

**Special case — Prefix/Postfix seq/qual length mismatch**: These compare two fields. Use the `Custom` spans pattern:
```rust
if let Some(seq) = self.seq.as_ref()
    && let Some(qual) = self.qual.as_ref()
    && seq.len() != qual.len()
{
    let spans = vec![
        (self.seq.span(), format!("{} characters", seq.len())),
        (self.qual.span(), format!("{} characters", qual.len())),
    ];
    self.seq.state = TomlValueState::Custom { spans };
    self.seq.help = Some("'seq' and 'qual' must be the same length".to_string());
}
```

### Phase 2: Move `validate_others` that need parent context (Medium)

These need `_input_def`, `_output_def`, or partial cross-transform context but the data is already available through the `PartialConfig` parent in `VerifyIn`.

| Transform | What it needs | How to access in `VerifyIn` |
|---|---|---|
| `Progress` | `output_def.stdout` | `parent.output` in `VerifyIn<PartialConfig>` |
| `StoreTagInComment` | output segment list | `parent.output` in `VerifyIn<PartialConfig>` |
| `Demultiplex` | upstream transform list (to check if `in_label` is Bool) | `parent.transform` in `VerifyIn<PartialConfig>` — but transforms are `PartialTransformation` at this stage. Need to check if tag type info is available. See **Complication** below. |

**Progress**: Straightforward — check `parent.output.as_ref()...stdout` and `self.output_infix`.

**StoreTagInComment**: Check the output segment list from `parent.output`. The segment validation is already similar to what other transforms do.

**Complication — Demultiplex and cross-transform checks**: `Demultiplex::validate_others` looks at upstream transforms' `declares_tag_type()` to determine if `in_label` produces a Bool tag. In the `VerifyIn` world, transforms are still `PartialTransformation` — we don't have the concrete `Transformation` yet, so `declares_tag_type()` (a `Step` trait method) isn't available.

**Options for Demultiplex**:
- (a) Keep it in `validate_others` for now (it's inherently a cross-transform check)
- (b) Add a method to `PartialTransformation` that can report declared tag types without full concretization — this requires modifying the tagged enum's generated code
- (c) Accept that this specific check stays post-deserialization

**Recommendation**: Option (a) — keep `Demultiplex::validate_others`. Same for `TrimAtTag` (needs to inspect upstream `ExtractRegions`), `OtherFileByName` (needs to inspect upstream `StoreTagInComment`), `StoreTagsInTable` (needs to check any upstream tag), and `ConcatTags` (partially — the `in_labels.len() < 2` check can move to `VerifyIn`, but the `all_transforms` inspection stays). `Report` duplicate name checking also needs the full transform list.

### Phase 3: Simple `Config::check_*` functions (Easy)

#### `check_benchmark` → `PartialConfig::verify` or `PartialBenchmark::verify`

Currently checks `molecule_count > 0`. Move to `VerifyIn`:
```rust
// In PartialBenchmark or PartialConfig::verify
self.molecule_count.verify(|v| {
    if *v == 0 {
        Err(ValidationFailure::new(
            "molecule_count must be > 0",
            Some("Set to a positive integer"),
        ))
    } else { Ok(()) }
});
```

Note: `Benchmark` currently has `#[tpd(no_verify)]`. Change to a proper `VerifyIn` impl. The `enable` field's side-effect (nullifying output) should stay in `check_benchmark` since it mutates `self.output`.

#### `check_for_any_output` → `PartialConfig::verify`

This checks that at least one output form exists. It cross-references output config and the transform list. Can be moved into `PartialConfig::verify` since it has access to all fields. Use `Custom` spans to highlight the output section when missing.

Note: it currently also checks for `StoreTagInFastQ`/`StoreTagsInTable`/`Inspect` in the stages list, which are post-expansion transforms. In `PartialConfig::verify`, you'd check `self.transform` for these variant types instead.

#### `check_head_rapidgzip_conflict` → `PartialConfig::verify`

Check if any transform is `Head` and `input.options.build_rapidgzip_index` is `Some(true)`. Both are accessible from `PartialConfig`. Use `Custom` spans pointing to both the Head step and the `build_rapidgzip_index` field.

#### `check_blocksize` → Already partially in `PartialOptions::verify`

The `block_size > 0` check is already there. The `block_size % 2 == 1 && interleaved` check needs access to input info. This is already handled in `PartialOptions::verify` via the parent context (`PartialConfig`). Verify it's complete; if not, add it.

### Phase 4: Medium-difficulty `Config::check_*` functions

#### `check_name_collisions` → `PartialConfig::verify`

This cross-references segment names, barcode names, and tag labels. Segment names and barcode names are available in `PartialConfig::verify`. Tag labels come from transforms (`declares_tag_type()`), which aren't available at the Partial stage.

**Approach**: Split into two parts:
- Segment-vs-barcode name collisions → move to `PartialConfig::verify` (both available)
- Tag-vs-segment and tag-vs-barcode collisions → keep in `check_name_collisions` (requires concrete transforms)

For the segment-vs-barcode part, use `Custom` spans to highlight both the colliding segment name and barcode name.

#### `check_input_duplicate_files` → `PartialInput::verify` or `PartialConfig::verify`

This operates on the `StructuredInput` which is built during `PartialInput::verify`. Since `StructuredInput` is already the concrete type, this check could be done right after `build_structured()` in `PartialInput::verify`.

**Caveat**: Currently `build_structured()` is called during `PartialInput::verify`. If duplicate file detection should happen at the same stage, add it there. The challenge is that `accept_duplicate_files` is on `Options`, not `Input`. You'd need to pass it through or check at the `PartialConfig` level.

**Recommendation**: Move to `PartialConfig::verify` where both `self.input` and `self.options` are accessible. After input verification succeeds, check for duplicate files.

### Phase 5: Hard — `check_input_format` (file I/O)

This function opens actual files to detect their format. It inherently cannot run during deserialization/verification because:
- Files might not exist (validation mode skips this)
- It does actual I/O
- It configures multithreading as a side-effect

**Recommendation**: Keep `check_input_format` as a post-deserialization check. However, improve its error messages:

1. Store file path spans during `PartialInput::verify` in a side table (e.g., `HashMap<PathBuf, Range<usize>>`)
2. In `check_input_format`, when an error occurs, look up the span and construct a pretty error that references the TOML source location
3. This requires threading the original TOML source through to `check_input_format`, or storing spans on the `Input` struct

This is optional polish and can be deferred.

### Phase 6: Hard — `check_transformations` (tag flow analysis)

This is the most complex check. It walks through transforms in order, tracking:
- Which tags are declared (name + type)
- Which tags are used (and type-checks them)
- Which tags are removed
- Warnings for unused tags

**This fundamentally requires ordered, concrete transforms** because:
- Tag types come from `declares_tag_type()` on `Step` trait
- Usage checking comes from `uses_tags()` on `Step` trait
- The check is order-dependent

**Recommendation**: Keep `check_transformations` post-deserialization. However:

1. **Move `validate_others` field-level checks into `VerifyIn`** (Phases 1-2 above) — this removes ~60% of the validation from `validate_others`, leaving only cross-transform checks
2. **Improve error spans**: For tag-related errors, the error currently says `[Step N (name)]: No step generating label 'foo'`. To get span precision:
   - Store the `in_label` TOML span on the concrete transform struct (add a `label_span: Range<usize>` field, populated during concretization from `MustAdapt`)
   - Use these spans to construct `TomlValueState::Custom` errors that highlight the exact `in_label = "foo"` location
   - This requires modest changes to the concretization path

This is the most impactful improvement but also the most invasive. It can be done incrementally.

---

## Implementation Order

1. **Phase 1** — Move simple `validate_others` into `VerifyIn` (8 transforms, ~1 hour)
2. **Phase 3** — Simple `check_*` functions: `check_benchmark`, `check_for_any_output`, `check_head_rapidgzip_conflict` (~30 min)
3. **Phase 2** — `Progress` and `StoreTagInComment` `validate_others` (~30 min)
4. **Phase 4** — `check_name_collisions` (segment-vs-barcode part), `check_input_duplicate_files` (~1 hour)
5. **Phase 5 & 6** — Optional polish for file I/O and tag flow errors (deferred)

## Testing

- After each phase, run `cargo test` to verify no regressions
- After each phase, run `dev/_update_tests.py` if test case outputs change (error messages will change format)
- Check that test cases with intentional errors still produce meaningful error messages
- Look for any `trybuild` `.stderr` snapshots that need updating: `TRYBUILD=overwrite cargo test`

## What stays in `check_*` (final state)

After all phases, the following remain in `Config::inner_check`:
- `check_transformations` — tag flow analysis (ordered, cross-transform, needs concrete types)
- `check_input_format` / `check_input_format_for_validation` — file I/O, runtime concern
- `configure_multithreading` — side-effect, not validation
- `configure_rapidgzip` — side-effect, not validation
- `check_benchmark` (the side-effect part: nullifying output when benchmark is on)
- Cross-transform `validate_others` calls within `check_transformations` (Demultiplex, TrimAtTag, OtherFileByName, StoreTagsInTable, Report, ConcatTags partial)
