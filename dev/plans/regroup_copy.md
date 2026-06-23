Goal: eliminate the `next_chunk` regroup copy — the per-read re-memcpy that
repackages the pod parser's per-decode-chunk emissions into exact
`block_size`-sized blocks — by deleting the regroup entirely. The pod parser
emits its native per-decode-chunk blocks (zero extra copy), and the combiner
matches segment block sizes by O(1) `StringPod`/`DualStringPod` *slicing*
instead of the parser pre-copying everything to a fixed size.

stringpod lives at /project/stringpod and is path-referenced for now.

  Why (profiling evidence, post-shm run, no-op downstream)

  Total CPU ≈ 295 s over the run. Breakdown of self-time:
  - ~50% gzip inflate in the decompressor process (irreducible decode).
  - ~25% libc AVX-512 memmove/memcpy — the copies.
  - ~22% fastqrab pod-parse code (newline scan + line split + pod build).
  - <1% downstream (read counting only — confirms parse is the whole cost).

  The 25% memmove splits into two call sites, both in the per-segment
  `Chunker_read*` thread pool:
  - COPY #1 — demux gather: slot bytes → per-decode-chunk columns
    (`demux_chunk` / `StringPodBuilder::push`). ~7% of total. *Unavoidable*:
    the shm slot is ephemeral (returned to the decompressor right after demux),
    so bytes must be copied into owned memory once.
  - COPY #2 — regroup: per-chunk columns → exact-`target` columns
    (`PodFastqParser::next_chunk` → `extend_from_pod`, the larger
    `DualStringPod` seq+qual buffer dominates). ~10% of total. **This is what
    we remove.** It exists only to repackage block sizes; it moves every byte a
    second time.

  Plus the decompressor's own 8% chunk→slot memcpy — left alone (Phase 2 of
  ipc.md, judged not worth it: not the gate, fights the recycle pool).

  The contract we have to keep

  The combiner pulls one block per segment and the downstream pairs read `i` of
  every segment, so the blocks it hands on must have *identical* read counts
  across segments (today asserted at pipeline.rs:478). R1 and R2 are independent
  gzip streams whose decode chunks hold different read counts, so something has
  to align them — today the parser does, by copying to a fixed `block_size`.

  Approach (Option D): make the combiner align by slicing, not the parser by
  copying. Each step the combiner slices every segment's current block down to
  the common `min(remaining)` and carries the remainders. Because it only ever
  slices *within* a single segment's current block (never joins two), every
  slice is a pure metadata operation on one shared `Arc` — no byte copy, ever,
  and no dependence on decode-chunk boundaries.

  The primitive: O(1) pod slicing

  The Region/Value StringPod redesign (`stringpod-tag-location-redesign`) is
  already implemented in /project/stringpod, so `slice` is built against the
  current model (the `Storage` FixedLength/Variable layout + `ColumnEdits` /
  `EditLog`). One new, cheap capability — slice a finished pod at record
  boundaries, sharing the `Arc` buffer, no bytes touched:

  - `StringPod::slice(&self, range: Range<usize>) -> StringPod`
    - FixedLength: O(1) — clone the `Arc`, set `count = range.len()`,
      `front_byte += range.start * stride`, keep stride/head_skip/visible_len.
      **Preserves the fixed-length fast path** (downstream cuts stay O(1)).
    - Variable: O(range) metadata only — clone the `Arc`, copy
      `positions[range]` (a `(u32,u32)` per entry, ~no bytes).
    - Visible content comes entirely from the shared `Storage` overlay, so a
      slice reads byte-identical to `iter().skip(start).take(len)`. Edit history
      (`ColumnEdits`/`EditLog`) is sliced to the range / kept consistent with
      the current `extend_from_pod` + `finish` semantics — confirm against
      `EditLog`.
  - `DualStringPod::slice(&self, range)` — same, sharing both byte `Arc`s and
    the single shared metadata column.
  - `FastQChunk::slice(&self, range)` in fastqrab-io/blocks.rs — slices `names`,
    `seq_quals`, and `pluses` (`new_all_empty(n)`) by the same range.

  This is the redesign's "COW alias pods" idea applied to block alignment.

  fastqrab work

  1. Parser — stop regrouping. `PodFastqParser::parse()` forwards the pod
     parser's native per-decode-chunk `FastqChunk` straight through (move
     `names`/`reads` in, `pluses = new_all_empty(count)`); delete the
     `pending` / `front_consumed` / `extend_from_pod` regroup. `target_reads_
     per_block` stops governing the emitted size (kept only as a builder hint /
     ignored). Blocks are now ~one decode chunk of reads each.
     - Other parsers (legacy `FastqParser`, BAM, FASTA) are unchanged: they may
       keep emitting fixed-size blocks. The new combiner is size-agnostic, so
       mixed/!= sizes across segments are handled uniformly (min just equals
       the common size when they already align).

  2. Combiner — `run_combiner_thread`: replace "pull one block per segment,
     assert equal length" with a slice-to-min zip carrying remainders:
     - Per-segment state: `Option<(FastQChunk, cursor)>` = current block and
       reads already consumed from it.
     - Each round: for every segment ensure a current block with reads
       remaining (recv next when `cursor == len`); let `n = min over segments of
       (len - cursor)`; emit `FastQBlocksCombined` of the per-segment
       `slice(cursor..cursor+n)`; advance every cursor by `n`.
     - EOF / count check: a segment is *done* when its channel is closed **and**
       its current block is fully consumed. If all segments are done at the top
       of a round → send the final empty block and return. If some are done
       while others still have reads → the existing "unequal number of reads"
       error. (This subsumes the old equal-length assert.)
     - Keep `block_no` / `first_read_in_block_no` / `expected_read_count`
       bookkeeping; `first_read_in_block_no += n` per emitted block.
     - The single-reader interleaved path (`split_interleaved`, pipeline.rs:367)
       is untouched — it already emits matched segments from one stream.

  3. (Optional, secondary) raise the decode/slot chunk size in
     `spawn_rapidgzip_shm` (still overridable via
     `FASTQRAB_DECOMP_SHM_SLOT_SIZE`). Not needed for correctness or
     copy-freeness under D; it just yields fewer, larger blocks (less per-block
     overhead) and fewer ragged small blocks at mis-aligned segment boundaries.
     Arbitrary value, and with millions of reads there are still plenty of
     parallel chunks, so this is free if we want it.

  Why D over the alternatives (considered, rejected)

  - Option B (parser-side slice + copy fallback at decode-chunk straddles,
    chunk-size lever): keeps exact `block_size` blocks, but a block straddling
    two decode-chunk `Arc`s still needs a copy, so it only removes the regroup
    *partially* (or relies on chunk >> target to make straddlers rare). More
    moving parts than D for a strictly smaller win.
  - Option A (segmented/rope columns so a block can span buffers zero-copy):
    always exact + always copy-free, but rewrites the core pod data model
    (`get`/`iter`/`Storage`/`DualStringPod` and every downstream consumer).
    High blast radius; not worth it when D is copy-free with just `slice`.

  D is the cleanest: copy-free always, only needs `slice`, simplifies the
  parser (the regroup loop is deleted, not relocated), and makes the combiner
  robust to any input block sizing.

  Implications of variable block sizes

  Blocks are no longer exactly `block_size`; sizes are the running cross-segment
  mins (≈ decode-chunk granularity, with occasional small remainder blocks where
  segment boundaries don't line up). This is correctness-neutral (the final
  block was already short, and steps don't require a fixed size), but it changes
  per-block overhead characteristics — verify no step implicitly assumes
  `block.len() == block_size`, and that very small remainder blocks don't add
  meaningful fixed-cost overhead (the optional chunk-size bump mitigates this).

  Memory: a sliced block shares its decode-chunk `Arc`, so a decode-chunk buffer
  stays resident until all its slices clear downstream — a few buffers pinned at
  once. Fine (same order as the shm slots already in flight).

  Phasing

  - Phase A — stringpod `slice` (+ `FastQChunk::slice`) and tests. Pure
    addition, no behavior change.
  - Phase B — combiner slice-to-min with carry + EOF/unequal handling, behind
    the existing combiner. Drive it first with the *current* fixed-size parser
    output (min == size, no remainders) so it's provably equivalent, then
    switch the pod parser to native blocks.
  - Phase C — delete `PodFastqParser::next_chunk` regroup; emit native blocks.
    Existing parser tests (pod_regroup_tests, rapidgzip cases, generated suite)
    must stay byte-identical end to end.
  - Phase D — optional decode/slot chunk-size bump; benchmark; confirm the
    COPY #2 site is gone in `samply`.

  Risks / things to verify

  - Fixed-length preservation through `slice` is essential — a slice that
    silently promotes to Variable would re-add O(n) cost downstream.
  - Combiner carry correctness: remainder slices must keep stream order and the
    record-pairing across segments; the "done = channel closed AND current block
    drained" condition must be exact so unequal-read-count inputs still error
    (cover first<later and first>later, matching today's messages).
  - `EditLog`/`ColumnEdits` behavior of `slice` matches the current
    `extend_from_pod` path (no liftover regression).
  - Downstream tolerance of variable / small block sizes (see above).

  Tests

  - stringpod: `slice` fixed & variable; equals `iter().skip().take()`;
    fixed-length stays fixed; slice under active `cut_start`/`cut_end` overlays;
    buffer shared (assert `buffer_bytes`/strong-count — no copy).
  - combiner: multi-segment with deliberately mismatched per-segment block
    sizes (e.g. different decode-chunk boundaries) → identical paired output and
    correct total counts; ragged tail (small remainders); unequal read counts
    error both directions; single-segment passthrough.
  - end-to-end: rapidgzip basic/single_thread + the generated suite stay
    byte-identical; the shm stress tests (tiny ring / multi-slot split) still
    pass with the native-block parser.

  Resolved (review answers)

  1. Approach: Option D — delete the regroup; parser emits native
     per-decode-chunk blocks; combiner aligns by slice-to-min with carry.
  2. Region/Value redesign is already implemented → `slice` is built against the
     current stringpod model.
  3. Decode/slot chunk-size bump is optional under D (a per-block-overhead
     tuning), not required; may be raised freely given the read volume.
