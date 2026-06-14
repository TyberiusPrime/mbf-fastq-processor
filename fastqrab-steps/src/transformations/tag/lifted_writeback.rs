//! Shared write-back machinery for [`StoreTagBackInSequence`] and
//! [`StoreTagInSequence`].
//!
//! Both steps splice a tag's content into a live read at a position described by
//! a *location* tag. The location's coordinates were captured in the **birth
//! frame** the tag was extracted in; the read has usually been edited since, so
//! the captured regions are first lifted into the read's current frame. A region
//! an edit cut away or split no longer has a home and is handled per [`OnLost`].
//!
//! An owned (divergent) location keeps references to *all* the original points its
//! content was taken from — it does not necessarily cover one contiguous span. A
//! write-back into such a target can only be made unambiguous in two cases:
//!
//! * the lifted regions form **one contiguous span** — then the whole span is
//!   replaced and the content may be longer or shorter (the read grows/shrinks);
//! * the regions are **disjoint but the content length matches** the total they
//!   cover — then the content is laid down byte-for-byte over exactly the covered
//!   positions, the bytes between the regions are left as they are, and the read
//!   keeps its length.
//!
//! Anything else (disjoint regions whose covered length differs from the content)
//! is a runtime error, as it was before the StringPod redesign.
//!
//! [`StoreTagBackInSequence`]: super::store_tag_back_in_sequence::StoreTagBackInSequence
//! [`StoreTagInSequence`]: super::store_tag_in_sequence::StoreTagInSequence

use crate::transformations::prelude::*;
use fastqrab_io::blocks::FastQChunk;
use stringpod::{DualStringPodMultiLocation, Lifted, OffsetLift, RegionLift};

/// What to do when a write-back's captured coordinates can no longer be located
/// in the read — the span was cut away or split by an edit applied *after* the
/// tag was captured, so there is nowhere to write.
#[derive(Clone, Copy, Debug, Default, JsonSchema)]
#[tpd]
pub enum OnLost {
    /// Silently leave the affected read unchanged (the default).
    Ignore,
    /// Abort the run with an error naming the affected reads.
    #[default]
    Complain,
}

/// Where a write-back lands, relative to the position tag's location. Both steps
/// funnel into the one code path; they differ only in which of these they pick.
pub(crate) enum WriteAnchor {
    /// Insert before the leftmost start of the location.
    Start,
    /// Insert after the rightmost end of the location.
    End,
    /// Overwrite the location's region(s) with the content.
    Replace,
}

/// One read's planned write-back, with coordinates still in the position tag's
/// birth frame (described by `born_generation` / `born_len`).
enum Target {
    /// Insert before birth-frame position `at` (`at >= born_len` ⇒ append at the
    /// read's current end). Length always changes freely — there is nothing to
    /// overwrite.
    InsertAt { at: usize },
    /// Overwrite the birth-frame `regions` (read-relative `(start, len)` pairs, in
    /// stored order). One region — or several that lift to a contiguous range —
    /// is a single span and accepts a length change; disjoint regions require the
    /// content to match their covered length.
    Replace { regions: Vec<(u32, u32)> },
}

struct Plan {
    born_generation: usize,
    born_len: usize,
    target: Target,
    seq: Vec<u8>,
    qual: Vec<u8>,
}

/// Build and apply the write-back for every read: take each row's content from
/// `value`, place it at `position`'s location per `anchor`, lift into the read's
/// current frame, and splice. `position` must be the location column; `value` is
/// the content source (its own `joined_seq`/`joined_qual` for a location tag, or
/// its bytes with `~` quality for a string tag). For `StoreTagBackInSequence` the
/// caller passes the *same* tag as both `position` and `value`.
pub(crate) fn store_tag_into_segment(
    segment: &mut FastQChunk,
    position: &DualStringPodMultiLocation,
    value: &TagColumn,
    anchor: WriteAnchor,
    on_lost: OnLost,
    step_name: &str,
) -> Result<()> {
    let n = position.row_count();
    let mut plans: Vec<Option<Plan>> = Vec::with_capacity(n);
    for row in 0..n {
        // The bytes to write, with their quality. A location value carries its
        // real (possibly edited) quality; a string value gets `~` (Phred 93).
        let content: Option<(Vec<u8>, Vec<u8>)> = match value {
            TagColumn::Location(vcol) => (!vcol.row_is_empty(row)).then(|| {
                (
                    vcol.joined_seq(row, None).to_vec(),
                    vcol.joined_qual(row, None).to_vec(),
                )
            }),
            TagColumn::String(items) => items.get_string(row).map(|v| {
                let seq = v.to_vec();
                let qual = vec![b'~'; seq.len()];
                (seq, qual)
            }),
            _ => panic!("{step_name}: value tag must be a Location or String column"),
        };
        let Some((seq, qual)) = content else {
            plans.push(None);
            continue;
        };
        // Nowhere to write.
        if position.row_is_empty(row) {
            plans.push(None);
            continue;
        }

        let (born_generation, born_len) = position.row_born(row);
        let regions: Vec<(u32, u32)> = position.row_regions(row).collect();
        let target = match anchor {
            WriteAnchor::Start => {
                let at = regions.iter().map(|&(s, _)| s).min().expect("non-empty") as usize;
                Target::InsertAt { at }
            }
            WriteAnchor::End => {
                let at = regions
                    .iter()
                    .map(|&(s, l)| s + l)
                    .max()
                    .expect("non-empty") as usize;
                Target::InsertAt { at }
            }
            WriteAnchor::Replace => Target::Replace { regions },
        };
        // Empty content inserts nothing — a true no-op; but empty content that
        // *replaces* a region deletes it (a shrinking edit), so only skip inserts.
        if seq.is_empty() && matches!(target, Target::InsertAt { .. }) {
            plans.push(None);
            continue;
        }
        plans.push(Some(Plan {
            born_generation,
            born_len,
            target,
            seq,
            qual,
        }));
    }

    apply_plans(segment, plans, on_lost, step_name)
}

/// Outcome of resolving one row's [`Plan`] against the read's current frame.
enum Resolved {
    /// A length-*changing* splice: replace `del` bytes at `at` with the content.
    /// Recorded into the edit log so later tag liftover shifts through it. Used
    /// for inserts and for replacing a single (contiguous) span with content of a
    /// different length.
    Splice {
        at: usize,
        del: usize,
        seq: Vec<u8>,
        qual: Vec<u8>,
    },
    /// A length-*neutral* overwrite of the covered positions (one `(pos, seq, qual)`
    /// per covered byte, in the read's current frame). Done in place and recorded
    /// as *nothing*: the coordinate space is unchanged, so a tag pointing at these
    /// bytes still validly points at them afterwards. This is "follow along the
    /// covered positions".
    InPlace(Vec<(usize, u8, u8)>),
    /// The captured location was cut away / split — handled per [`OnLost`].
    Lost,
    /// Disjoint regions whose covered length differs from the content — a hard
    /// error regardless of [`OnLost`].
    Invalid,
}

fn apply_plans(
    segment: &mut FastQChunk,
    plans: Vec<Option<Plan>>,
    on_lost: OnLost,
    step_name: &str,
) -> Result<()> {
    let n = plans.len();
    assert_eq!(
        segment.seq_quals.len(),
        n,
        "{step_name}: tag column ({}) and segment ({}) row counts must match",
        n,
        segment.seq_quals.len(),
    );

    // Pass 1: lift each plan into the read's current frame (borrows the segment
    // immutably, so it must finish before any mutation).
    let mut splices: Vec<Option<(usize, usize, Vec<u8>, Vec<u8>)>> = vec![None; n];
    let mut in_place: Vec<(usize, Vec<(usize, u8, u8)>)> = Vec::new();
    // Keep the plan of each lost row so the error can show a concrete example.
    let mut lost: Vec<(usize, Plan)> = Vec::new();
    let mut invalid: Vec<usize> = Vec::new();
    for (row, plan) in plans.into_iter().enumerate() {
        let Some(plan) = plan else { continue };
        match resolve(segment, row, &plan) {
            Resolved::Splice { at, del, seq, qual } => splices[row] = Some((at, del, seq, qual)),
            Resolved::InPlace(writes) => {
                if !writes.is_empty() {
                    in_place.push((row, writes));
                }
            }
            Resolved::Lost => lost.push((row, plan)),
            Resolved::Invalid => invalid.push(row),
        }
    }

    // Disjoint, length-changing targets can't be placed unambiguously — bail
    // (this is the case that errored before the StringPod redesign too).
    if !invalid.is_empty() {
        bail!(
            "{step_name}: the write-back target of {} read(s) covers multiple disjoint regions \
             whose total length differs from the replacement content, so it cannot be applied \
             unambiguously (rows {invalid:?}). Make the target a single span, or keep the \
             content the same length as the regions it replaces.",
            invalid.len(),
        );
    }
    if matches!(on_lost, OnLost::Complain) && !lost.is_empty() {
        let rows: Vec<usize> = lost.iter().map(|(row, _)| *row).collect();
        let (example_row, example_plan) = &lost[0];
        let example = describe_lost(segment, *example_row, example_plan);
        bail!(
            "{step_name}: the captured location of {} read(s) was lost to edits applied after \
             the tag was captured (rows {rows:?}).\n  For example, {example}\n\
             Set on_lost = 'Ignore' to skip these reads instead.",
            rows.len(),
        );
    }

    // Pass 2a: length-neutral overwrites, in place. `make_exclusive` detaches the
    // read buffers from any tag column COW-aliasing them, so the writes land
    // without disturbing those frozen snapshots — and without recording an edit,
    // so coordinates (and any tag still pointing into these bytes) are preserved.
    if !in_place.is_empty() {
        segment.seq_quals.make_exclusive();
        for (row, writes) in &in_place {
            let (s, q) = segment
                .seq_quals
                .pair_mut(*row)
                .expect("buffers made exclusive above; row in range. Bug");
            let s: &mut [u8] = s;
            let q: &mut [u8] = q;
            for &(pos, sb, qb) in writes {
                s[pos] = sb;
                q[pos] = qb;
            }
        }
    }

    // Pass 2b: length-changing splices, recording each length change so later tag
    // liftover shifts through it. Rows touched in place above are `None` here, so
    // their (already written) bytes are carried across the rebuild unchanged.
    if splices.iter().any(Option::is_some) {
        segment.seq_quals.splice_entries(&splices);
    }
    Ok(())
}

/// Build a one-line, human-readable explanation of why `row`'s captured location
/// could not be lifted into the read's current frame: which read, where the tag
/// was captured (in the read's *birth* frame), and the edits applied since that
/// removed or split those bases. Used to give the `OnLost::Complain` error a
/// concrete example instead of a bare row index.
fn describe_lost(segment: &FastQChunk, row: usize, plan: &Plan) -> String {
    let name = segment.names.get(row);
    let edits = match segment.seq_quals.ops_since(plan.born_generation, row) {
        Ok(view) => view.to_string(),
        Err(_) => "(edit history unavailable)".to_string(),
    };
    let captured = match &plan.target {
        Target::Replace { regions } => {
            let regs: Vec<String> = regions
                .iter()
                .map(|&(start, len)| format!("{start}..{}", start + len))
                .collect();
            format!("covering {}", regs.join(", "))
        }
        Target::InsertAt { at } => format!("anchored at {at}"),
    };
    format!(
        "read '{name}' had its tag {captured} in a {born_len}bp read, but the edits applied \
         since ({edits}) removed or split those bases, leaving nowhere to write it back",
        born_len = plan.born_len,
    )
}

/// Lift one row's [`Plan`] into the read's current frame, classifying it into an
/// in-place overwrite, a length-changing splice, a lost location, or an invalid
/// (ambiguous) target.
fn resolve(segment: &FastQChunk, row: usize, plan: &Plan) -> Resolved {
    let view = segment
        .seq_quals
        .ops_since(plan.born_generation, row)
        .expect("born generation captured from this pod; row in range. Bug");

    let regions = match &plan.target {
        Target::InsertAt { at } => {
            // A single insertion point: pure length change, always a splice.
            let pos = if *at >= plan.born_len {
                Some(view.current_len(plan.born_len))
            } else {
                match view.map_position(*at, plan.born_len) {
                    Ok(OffsetLift::At(pos)) => Some(pos),
                    Ok(OffsetLift::Deleted) | Err(_) => None,
                }
            };
            return match pos {
                Some(at) => Resolved::Splice {
                    at,
                    del: 0,
                    seq: plan.seq.clone(),
                    qual: plan.qual.clone(),
                },
                None => Resolved::Lost,
            };
        }
        Target::Replace { regions } => regions,
    };

    // Lift every region; if any was cut away or split, the whole target is lost.
    // A zero-width region (e.g. a `$` regex anchor that grows content from nothing)
    // is a single insertion *point*, lifted via `map_position` — `map_region`
    // rejects an empty region.
    let mut lifted: Vec<(usize, usize)> = Vec::with_capacity(regions.len());
    for &(start, len) in regions {
        if len == 0 {
            // A point at (or past) the birth-frame end appends at the read's
            // current end; `map_position` only handles interior positions.
            let pos = if start as usize >= plan.born_len {
                view.current_len(plan.born_len)
            } else {
                match view.map_position(start as usize, plan.born_len) {
                    Ok(OffsetLift::At(pos)) => pos,
                    Ok(OffsetLift::Deleted) | Err(_) => return Resolved::Lost,
                }
            };
            lifted.push((pos, 0));
        } else {
            match view.map_region(start as usize, len as usize, plan.born_len) {
                Ok(RegionLift::Kept { start, len }) => lifted.push((start, len)),
                Ok(RegionLift::Dropped) | Err(_) => return Resolved::Lost,
            }
        }
    }
    if lifted.is_empty() {
        return Resolved::Lost;
    }
    lifted.sort_unstable_by_key(|&(start, _)| start);

    let min_start = lifted.first().expect("non-empty").0;
    let max_end = lifted.iter().map(|&(s, l)| s + l).max().expect("non-empty");
    let covered_len: usize = lifted.iter().map(|&(_, l)| l).sum();
    let bounding_len = max_end - min_start;

    if plan.seq.len() == covered_len {
        // Unchanged length ⇒ follow along the covered positions, in place. Content
        // maps 1:1 onto the covered bytes in order; any gaps between regions are
        // left untouched. No coordinate change, so nothing is recorded.
        let mut writes = Vec::with_capacity(covered_len);
        let mut taken = 0;
        for &(start, len) in &lifted {
            for pos in start..start + len {
                writes.push((pos, plan.seq[taken], plan.qual[taken]));
                taken += 1;
            }
        }
        return Resolved::InPlace(writes);
    }

    // Length changes. That is only unambiguous over a single contiguous span;
    // disjoint regions with a differing length are a hard error.
    if bounding_len == covered_len {
        Resolved::Splice {
            at: min_start,
            del: bounding_len,
            seq: plan.seq.clone(),
            qual: plan.qual.clone(),
        }
    } else {
        Resolved::Invalid
    }
}
