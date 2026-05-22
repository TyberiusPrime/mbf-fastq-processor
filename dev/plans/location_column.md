# Compact `LocationColumn` storage

Goal: replace `TagColumn::Location(Vec<Option<Hits>>)` — currently 2 heap allocations and ~150 bytes per single-hit slot, plus a `BString` per hit — with a flat byte slab plus 16-byte `Copy` `Hit` records, single inline SmallVec slot. Hand-rolled slab; no new crate dependency.

## Baseline (today)

`fastqrab-dna/src/dna.rs:17-40`:

```rust
pub struct HitRegion { start: usize, len: usize, segment_index: SegmentIndex }   // 24 B
pub struct Hit { location: Option<HitRegion>, sequence: BString }                 // 56 B + heap
pub struct Hits(pub Vec<Hit>);                                                    // 24 B + heap
pub enum TagColumn { Location(Vec<Option<Hits>>), ... }
```

`fastqrab-dna/src/segments.rs:5`: `SegmentIndex(pub usize)`.

Per single-hit slot: `Option<Hits>` 24 B (niche-opt) + `Vec<Hit>` heap 24+ B + `Hit` 56 B + `BString` heap 24 + N B. **2 allocations.**

## Target design

### `SegmentIndex` shrinks to `u8`

(already done)


### New `Hit` (16 B, `Copy`)

```rust
// fastqrab-dna/src/dna.rs
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub seq_start: u32,     // offset into LocationColumn slab
    pub loc_start: u32,     // start in read (valid iff flags & HAS_LOC)
    pub seq_len: u16,       // bytes in slab
    pub loc_len: u16,       // length in read (valid iff flags & HAS_LOC)
    pub segment_index: u8,  // valid iff flags & HAS_LOC
    pub flags: u8,
    _pad: [u8; 2],
}
pub const HAS_LOC: u8 = 0b0000_0001;

// View used at call boundaries (filter_tag_locations callbacks, etc.)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitRegionView {
    pub start: usize,
    pub len: usize,
    pub segment_index: SegmentIndex,
}
```

`Hits` = `SmallVec<[Hit; 1]>` (24 B). **Empty SmallVec ≡ None** — no `Option` wrapper, no discriminant byte. Single-hit hot path stays fully inline (no heap).

Width justifications:
- `seq_start: u32` — 4 GiB slab per column per block; blocks are bounded.
- `seq_len: u16` — barcodes/extracted regions; assert at push, return error if exceeded.
- `loc_start: u32`, `loc_len: u16` — reads < 4 GiB, regions < 65 k.
- `segment_index: u8` — pipelines have ≤ 255 segments; enforced in config.

### `LocationColumn` (new inner type)

`TagColumn::Location` wraps this; `TagColumn` methods delegate.

```rust
// fastqrab-dna/src/dna.rs
pub struct LocationColumn {
    slab: Vec<u8>,
    hits: Vec<Hits>,
}

impl LocationColumn {
    pub fn new() -> Self;
    pub fn with_capacity(slots: usize, slab_bytes: usize) -> Self;

    // --- builders ---
    pub fn push_none(&mut self);
    pub fn push_single(&mut self, loc: Option<HitRegionView>, seq: &[u8]);
    pub fn push_many(&mut self, entries: &[(Option<HitRegionView>, &[u8])]);
    /// Replace slot; new bytes appended to slab (old bytes go dead until compact).
    pub fn set_slot(&mut self, idx: usize, entries: &[(Option<HitRegionView>, &[u8])]);
    pub fn clear_slot(&mut self, idx: usize);

    // --- accessors ---
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn get(&self, idx: usize) -> &Hits;
    pub fn iter(&self) -> impl Iterator<Item = &Hits>;
    pub fn hit_bytes(&self, h: &Hit) -> &[u8];
    /// Length-preserving mutation only (rev-comp, case change).
    pub fn hit_bytes_mut(&mut self, h: &Hit) -> &mut [u8];
    pub fn hit_location(&self, h: &Hit) -> Option<HitRegionView>;
    pub fn set_hit_location(&mut self, slot: usize, hit_idx: usize, loc: Option<HitRegionView>);

    // --- bulk ops ---
    pub fn resize_with_empty(&mut self, len: usize);  // panic on shrink (matches today)
    pub fn retain<F: FnMut(usize) -> bool>(&mut self, f: F);
    pub fn drain(&mut self, range: std::ops::Range<usize>);
    /// Append other's slots, translating seq_start by self.slab.len() before the copy.
    pub fn extend_from(&mut self, other: &LocationColumn);

    // --- views / joins (replace old Hits methods) ---
    pub fn joined_sequence(&self, hits: &Hits, sep: Option<&[u8]>) -> Vec<u8>;
    pub fn joined_sequence_cow<'a>(&'a self, hits: &'a Hits, sep: Option<&[u8]>) -> Cow<'a, [u8]>;
    pub fn covered_len(&self, hits: &Hits) -> usize;
    pub fn location_str(&self, hits: &Hits, segment_order: &[String]) -> BString;

    /// Drop unreferenced slab bytes; rewrite seq_start.
    /// Two pass: collect referenced (start,len) sorted, copy contiguous, build offset map.
    pub fn compact_slab(&mut self);
}
```

### `TagColumn`

```rust
pub enum TagColumn {
    Location(LocationColumn),
    String(Vec<Option<BString>>),
    Numeric(Vec<f64>),
    Bool(Vec<bool>),
}
```

Reimplement every method on `TagColumn` (current `dna.rs:42-236`) as thin delegates:
- `new_empty`, `resize_with`, `len`, `is_empty`, `retain`, `drain`, `extend` → delegate (only if already present!).
- `into_locations` → `Option<LocationColumn>` (was `Vec<Option<Hits>>`). Callers update.
- `as_locations` → `Option<&LocationColumn>`.
- `iter_locations` → returns `&LocationColumn` (callers can both iterate `.iter()` and resolve bytes).
- `iter_stringified`, `iter_truthy`, `to_bstr`, `get_location` → use `LocationColumn::joined_sequence_cow` for byte resolution.

Per single-hit slot after: 24 B (SmallVec inline) + N B in shared slab + **0 allocations**.

## Order of work

Each step ships green before moving on.

### Step 1 — `SegmentIndex` → `u8`

Solved.

### Step 2 — New types in `fastqrab-dna`

2. In `fastqrab-dna/src/dna.rs`:
   - Delete old `HitRegion`, `Hit`, `Hits`.
   - Add new `Hit`, `HAS_LOC`, `HitRegionView`, `type Hits = SmallVec<[Hit; 1]>`.
   - Add `LocationColumn` struct + impls per "Target design".
   - Change `TagColumn::Location(Vec<Option<Hits>>)` → `TagColumn::Location(LocationColumn)`.
   - Reimplement all `TagColumn` methods as delegates.
3. Update `find_iupac`, `find_iupac_with_indel` in `fastqrab-dna/src/dna.rs:345-505`:
   - They cannot return `Option<Hits>` (no slab in scope). New return type:
     ```rust
     pub struct HitDraft {
         pub location: Option<HitRegionView>,
         pub sequence: Vec<u8>,
     }
     ```
   - Callers feed `HitDraft` into `LocationColumn::push_single`.
4. Update the test module (`dna.rs:921+`) to build a `LocationColumn`, push, and assert via `column.hit_bytes(&hit)` and `Hit` fields. Test helper:
   ```rust
   fn one_hit(start: u32, len: u16, seg: u8, seq: &[u8]) -> (LocationColumn, Hit) {
       let mut col = LocationColumn::new();
       col.push_single(Some(HitRegionView { start: start as usize, len: len as usize, segment_index: SegmentIndex(seg) }), seq);
       let hit = col.get(0)[0];
       (col, hit)
   }
   ```

### Step 3 — Re-export shim

`fastqrab-steps/src/transformations/prelude.rs:13`: keep names `Hit`, `Hits`, `TagColumn` exported; export `LocationColumn`, `HitRegionView`. If anything outside the workspace re-exports `HitRegion`, alias `pub use HitRegionView as HitRegion;` for now.

### Step 4 — Migrate constructors

Replace `Hits::new(...)`, `Hits::new_multiple(...)`, and `TagColumn::Location(vec)` patterns. Build a `LocationColumn` directly; insert via `TagColumn::Location(col)`.

Files (each independent; do one at a time, run tests):

- `fastqrab-steps/src/transformations/extract.rs:53,90`
- `fastqrab-steps/src/transformations/extract/regions.rs:200-221`
- `fastqrab-steps/src/transformations/extract/regions_of_low_quality.rs:88-121`
- `fastqrab-steps/src/transformations/extract/iupac.rs:127-149`
- `fastqrab-steps/src/transformations/extract/iupac_suffix.rs:120`
- `fastqrab-steps/src/transformations/extract/low_quality_start.rs:69`
- `fastqrab-steps/src/transformations/extract/low_quality_end.rs:69`
- `fastqrab-steps/src/transformations/extract/poly_tail.rs:99`
- `fastqrab-steps/src/transformations/extract/longest_poly_x.rs:350`
- `fastqrab-steps/src/transformations/extract/regex.rs:240`
- `fastqrab-steps/src/transformations/hamming_correct.rs:463-808` (creates corrected hits → new `LocationColumn`)
- `fastqrab-steps/src/transformations/tag/concat_tags.rs:185-208`
- `fastqrab-steps/src/transformations/edits/trim_at_tag.rs:208`

### Step 5 — Migrate readers

Sites reading `hits.0`, `hit.sequence`, `hit.location` — now go through the owning `LocationColumn`:

- `fastqrab-steps/src/transformations.rs:468-508` (filter_tag_locations dispatch — central, do first)
- `fastqrab-steps/src/transformations/filters/reservoir_sample.rs:121,137`
- `fastqrab-steps/src/transformations/hamming_exact_counter.rs:174-179`
- `fastqrab-steps/src/transformations/hamming_correct.rs:463-808` (also writer)
- `fastqrab-steps/src/transformations/tag/concat_tags.rs:197,217`
- `fastqrab-steps/src/transformations/tag/store_tag_in_comment.rs:177`
- `fastqrab-steps/src/transformations/tag/store_tag_in_fastq.rs:230-267`
- `fastqrab-steps/src/transformations/tag/store_tag_in_sequence.rs:103-215`
- `fastqrab-steps/src/transformations/tag/store_tag_back_in_sequence.rs:60-124`
- `fastqrab-steps/src/transformations/tag/store_tags_in_table.rs:259`
- `fastqrab-steps/src/transformations/tag/store_single_cell_matrix.rs:481-512`
- `fastqrab-steps/src/transformations/tag/quantify_tag.rs:127-142`
- `fastqrab-steps/src/transformations/tag/replace_tag_with_letter.rs:44-45`
- `fastqrab-steps/src/transformations/tag/assign_by_halves.rs:137-156`
- `fastqrab-steps/src/transformations/tag/fill_missing.rs:102-129`
- `fastqrab-steps/src/transformations/reports/report_tag_histogram.rs:55-60`
- `fastqrab-steps/src/transformations/convert/regions_to_length.rs:72-84`
- `fastqrab-steps/src/transformations/validation/all_reads_same_length.rs:98-101`
- `fastqrab-steps/src/transformations/calc/worst_quality.rs:88`
- `fastqrab-steps/src/transformations/convert/eval_expression.rs:241`

In-place sequence mutations (length-preserving) — use `LocationColumn::hit_bytes_mut`:

- `fastqrab-steps/src/transformations/edits/_change_case.rs:115-121`
- `fastqrab-steps/src/transformations/edits/reverse_complement.rs:110-117`

Location clearing — clear `flags & !HAS_LOC` via `LocationColumn::set_hit_location(..None)`:

- `fastqrab-steps/src/transformations/edits/trim_at_tag.rs:96-202`

### Step 6 — Compaction

Implement `LocationColumn::compact_slab` and call it where measurements justify the work:
- end of `hamming_correct` after rebuilding the column,
- end of any `extract` step with a high prune rate.

Skip the call sites if benches don't show wins; the function is still useful for the user to invoke manually.

### Step 7 — Benches & cleanup

- New criterion bench in `fastqrab-dna/benches/`: build 1 M single-hit slots, before/after.
- Run full integration suite.
- Remove `BString` import in `dna.rs` if unused after migration.
- Verify with `cargo asm` (or by reading the `Vec<Hits>` Drop) that empty SmallVec drop avoids the heap branch — should already be the case but worth checking once.

## Validation guards

- `push_single` / `push_many` / `set_slot`: `debug_assert!(seq.len() <= u16::MAX as usize)` and a real `bail!` in release if exceeded. Same for `loc.len`, `loc.start` ≤ `u32::MAX as usize`.
- `SegmentIndex::try_from` already errors above `MAX_SEGMENTS`.
- `hit_bytes_mut`: doc-comment "length-preserving only"; no runtime check needed.

## Risks

- **In-place mutation aliasing.** `hit_bytes_mut` assumes each `Hit` owns a unique slab range. True at construction. Add a test that re-uses the same `&[u8]` source for two `push_single` calls and checks that mutating one does *not* affect the other (i.e. confirm we always *copy* into the slab, never share offsets).
- **`extend_from` offset translation.** Add a round-trip test: build A, build B with a hit, `A.extend_from(&B)`, assert bytes resolved through A match what B had.
- **Removed `HitRegion` type.** External crates (if any) re-importing `HitRegion` break — the prelude alias mitigates, but greppable.
- **`SmallVec::Drop` for empty.** Confirm it's a no-op branch (cheap). If profiling shows otherwise, switch to `tinyvec::TinyVec` or a hand-rolled `OneOrMany<Hit>`.

## Expected wins

- Per single-hit slot: **~150 B + 2 allocs → 24 B + N slab bytes + 0 allocs.**
- Per `Hit`: **56 B → 16 B** (3.5× smaller, `Copy`).
- `SegmentIndex`: 8 B → 1 B (mostly absorbed by `Hit` packing, but cleans up `HitRegionView` size too).
- Lower allocator pressure on hot extract / hamming-correct paths; better cache density when iterating hits.
