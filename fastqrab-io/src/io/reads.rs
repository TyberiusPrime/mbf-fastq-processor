use anyhow::{Result, bail};
use bstr::BStr;
use indexmap::IndexMap;
use smallvec::SmallVec;
use std::marker::PhantomData;
use std::num::NonZero;
use std::ops::Range;
use stringpod::CrossPods;

use crate::blocks::{self, FastQChunk, FastQReadMut, Molecules, MoleculesMut};
use fastqrab_config::{TagLabel, segments::SegmentIndexOrAll};
use fastqrab_dna::dna::{TagColumn, hamming};
use fastqrab_dna::segments::SegmentIndex;

pub type DemultiplexTag = u64;
pub type Tags = IndexMap<TagLabel, TagColumn>;
/// Read in memory representation.
/// We either have references in the large block we read from the fastq file,
/// or owned sections. We therefore need to pass in the block as an 'arena' when
/// accessing sequencing data. Benefit is the zero-copy parsing and handling of fastq data.
///
/// We hide this complexity from consumers behind `WrappedFastQRead` and `WrappedFastQReadMut`,
///
/// This module also has higher level functions to work on blocks of fastq reads.

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub start: usize,
    pub end: usize,
}
// we either store the read parts in their own Vec<u8>
// *or* as positions in a larger buffer.
// and the parser places *most* reads in the buffer,
// greatly reducing the number of allocations we do.

pub struct SegmentsCombined<T> {
    pub segments: Vec<T>,
}

/// Multiple fastqblocks together with their tag annotation
/// and output destination.
///
/// Contract: segments must not be empty.
#[derive(Clone, Debug)]
pub struct FastQBlocksCombined {
    /// One segment per read mate (R1, R2, index, ...), kept in lockstep: read
    /// `i` of every segment belongs to the same molecule. The lockstep invariant
    /// (every segment has the same read count) is established at construction and
    /// preserved by the count-changing methods ([`truncate`](Self::truncate) /
    /// [`drain`](Self::drain) / [`apply_bool_filter`](Self::apply_bool_filter)).
    /// Mutate a single segment's reads via [`member_mut`](Self::member_mut),
    /// which re-checks the invariant on drop.
    pub segments: Vec<FastQChunk>, //todo: Make private, rename molecules?
    pub output_tags: Option<Vec<DemultiplexTag>>, // used by Demultiplex
    pub tags: IndexMap<TagLabel, TagColumn>,
    pub is_final: bool,
    block_no: usize,
    molecules_at_start: usize,
    pub first_read_sequential_number: usize,
    _force_private: PhantomData<u8>,
}

impl FastQBlocksCombined {
    /// # Panics
    /// - if `segments` is empty
    /// - if the segments are not in lockstep (differ in read count)
    #[must_use]
    pub fn new(
        segments: Vec<FastQChunk>,
        output_tags: Option<Vec<DemultiplexTag>>,
        tags: IndexMap<TagLabel, TagColumn>,
        is_final: bool,
        block_no: usize,
        first_read_sequential_number: usize,
    ) -> Self {
        assert!(
            !segments.is_empty(),
            "Empty segments not supported in FastQBlocksCombined"
        );
        let expected = segments[0].row_count();
        for (idx, segment) in segments.iter().enumerate().skip(1) {
            assert_eq!(
                segment.row_count(),
                expected,
                "FastQBlocksCombined segment {idx} has {} reads but segment 0 has {expected}; \
                 segments must be in lockstep",
                segment.row_count(),
            );
        }
        FastQBlocksCombined {
            molecules_at_start: segments[0].len(),
            segments,
            output_tags,
            tags,
            is_final,
            block_no,
            first_read_sequential_number,
            _force_private: PhantomData,
        }
    }

    #[must_use]
    pub fn block_no(&self) -> usize {
        self.block_no
    }

    /// Read count before the block was processed at all
    ///
    /// Necessary to count reads-in-flight correctly
    #[must_use]
    pub fn initial_molecule_count(&self) -> usize {
        self.molecules_at_start
    }

    #[must_use]
    pub fn iter_segment_indices(&self, idx: SegmentIndexOrAll) -> Vec<usize> {
        match idx {
            SegmentIndexOrAll::All => (0..self.member_count()).collect(),
            SegmentIndexOrAll::Indexed(idx) => vec![idx.as_index()],
        }
    }

    #[must_use]
    pub fn iter_matching_segments<'a>(
        &'a self,
        idx: SegmentIndexOrAll,
    ) -> Box<dyn Iterator<Item = &'a FastQChunk> + 'a> {
        match idx {
            SegmentIndexOrAll::All => Box::new(self.segments.iter()),
            SegmentIndexOrAll::Indexed(query_index) => Box::new(
                self.segments
                    .iter()
                    .enumerate()
                    .filter_map(move |(idx, segment)| {
                        if idx == query_index.as_index() {
                            Some(segment)
                        } else {
                            None
                        }
                    }),
            ),
        }
    }

    #[must_use]
    pub fn iter_matching_segments_mut<'a>(
        &'a mut self,
        idx: SegmentIndexOrAll,
    ) -> Box<dyn Iterator<Item = &'a mut FastQChunk> + 'a> {
        match idx {
            SegmentIndexOrAll::All => Box::new(self.segments.iter_mut()),
            SegmentIndexOrAll::Indexed(query_index) => Box::new(
                self.segments
                    .iter_mut()
                    .enumerate()
                    .filter_map(move |(idx, segment)| {
                        if idx == query_index.as_index() {
                            Some(segment)
                        } else {
                            None
                        }
                    }),
            ),
        }
    }
    /// create an empty one with the same options filled, and same `block_no`
    #[must_use]
    pub fn empty(&self) -> FastQBlocksCombined {
        FastQBlocksCombined {
            segments: vec![FastQChunk::new_empty(); self.member_count()],
            output_tags: if self.output_tags.is_some() {
                Some(Vec::new())
            } else {
                None
            },
            tags: IndexMap::default(),
            is_final: self.is_final,
            block_no: self.block_no,
            first_read_sequential_number: self.first_read_sequential_number,
            molecules_at_start: self.molecules_at_start,
            _force_private: PhantomData,
        }
    }

    #[must_use]
    pub fn with_new_block_no(&self, block_no: usize) -> FastQBlocksCombined {
        let mut res = self.clone();
        res.block_no = block_no;
        res
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.row_count()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.row_count() == 0
    }

    /// Number of molecules — the shared read count across all segments.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.segments.first().map_or(0, FastQChunk::row_count)
    }

    /// Number of segments (mates) per molecule.
    #[must_use]
    pub fn member_count(&self) -> usize {
        self.segments.len()
    }

    /// Read-only access to one segment.
    ///
    /// # Panics
    /// If `index >= self.member_count()`.
    #[must_use]
    pub fn member(&self, index: usize) -> &FastQChunk {
        &self.segments[index]
    }

    /// Mutable access to one segment, for in-place work that **keeps the read
    /// count unchanged**. The returned guard re-checks this segment's read count
    /// on drop and panics if it drifted out of lockstep — count changes must go
    /// through [`truncate`](Self::truncate) / [`drain`](Self::drain) /
    /// [`apply_bool_filter`](Self::apply_bool_filter).
    ///
    /// # Panics
    /// If `index >= self.member_count()`.
    #[must_use]
    pub fn member_mut(&mut self, index: usize) -> MemberGuard<'_> {
        let expected = self.segments[index].row_count();
        MemberGuard {
            segment: &mut self.segments[index],
            expected,
        }
    }

    /// Swap two segments (mates). Reorders only — every segment keeps its read
    /// count, so the lockstep invariant is preserved.
    ///
    /// # Panics
    /// If either index is out of range.
    pub fn swap_members(&mut self, a: usize, b: usize) {
        self.segments.swap(a, b);
    }

    /// Iterate molecules: each item is read `i` drawn from every segment, in
    /// segment order.
    #[must_use]
    pub fn molecules(&self) -> Molecules<'_> {
        blocks::molecules(&self.segments)
    }

    /// Iterate molecules mutably. Shared buffers are made exclusive first
    /// (silent copy-on-write).
    pub fn molecules_mut(&mut self) -> MoleculesMut<'_> {
        blocks::molecules_mut(&mut self.segments)
    }

    /// Ensure every segment owns its byte buffers outright.
    pub fn make_exclusive(&mut self) {
        for segment in &mut self.segments {
            segment.make_exclusive();
        }
    }

    pub fn truncate(&mut self, len: usize) {
        for segment in &mut self.segments {
            segment.truncate(len);
        }
        if let Some(output_tags) = &mut self.output_tags {
            output_tags.truncate(len);
        }
        for tags in self.tags.values_mut() {
            tags.truncate(len);
        }
    }

    /// # Panics
    /// when used on demultiplexed blocks
    pub fn drain(&mut self, range: Range<usize>) {
        assert!(
            self.output_tags.is_none(),
            "Drain used on a demultiplexed block. I don't think that's sensible"
        );
        for segment in &mut self.segments {
            segment.drain(range.clone());
        }
        // if let Some(output_tags) = &mut self.output_tags {
        //     output_tags.drain(range.clone()); // cov:excl-line currently not being used, since we
        //     // only use drain in the non-demultiplexing case, completeness and future use.
        // }
    }

    ///in place mutation.
    pub fn apply_mut_sequences<F>(&mut self, f: F)
    where
        F: for<'a> Fn(&SmallVec<[&'a mut BStr; 4]>),
    {
        // One molecule per row; gather each segment's seq into the slice the
        // closure expects. `molecules_mut` makes the buffers exclusive itself
        // (silent copy-on-write), so the old "apply to_exclusive first" caveat
        // is gone.
        for molecule in blocks::molecules_mut(&mut self.segments) {
            let mut reads: SmallVec<[&mut BStr; 4]> =
                molecule.into_iter().map(|read| read.seq).collect();
            f(&mut reads);
        }
    }

    ///in place mutati
    pub fn apply_mut_qualities<F>(&mut self, f: F)
    where
        F: for<'a> Fn(&mut SmallVec<[&'a mut BStr; 4]>),
    {
        for molecule in blocks::molecules_mut(&mut self.segments) {
            let mut reads: SmallVec<[&mut BStr; 4]> =
                molecule.into_iter().map(|read| read.qual).collect();
            f(&mut reads);
        }
    }

    ///in place mutation.
    pub fn apply_mut_both<F>(&mut self, f: F)
    where
        F: for<'a> Fn(&mut SmallVec<[(&'a mut BStr, &'a mut BStr); 4]>),
    {
        for molecule in blocks::molecules_mut(&mut self.segments) {
            let mut reads: SmallVec<[(&mut BStr, &mut BStr); 4]> = molecule
                .into_iter()
                .map(|read| (read.seq, read.qual))
                .collect();
            f(&mut reads);
        }
    }

    /// # Panics
    /// when the tag is missing or not a Location column
    pub fn apply_mut_with_location_tag<F>(&mut self, label: &TagLabel, mut f: F)
    where
        F: for<'a> FnMut(&mut BStr, &SmallVec<[(u32, u32); 1]>),
    {
        let TagColumn::Location(col) = self.tags.get(label).expect("Tag must be present, bug")
        else {
            panic!("Tag {label:?} is not a Location column");
        };
        // // `col` borrows `self.tags`; `molecules()` borrows `self.segments` — two
        // // disjoint, immutable field borrows. The closure only sees `&BStr`, so
        // // read-only molecule iteration is enough (no copy-on-write needed).
        for (seq, row_regions) in self.segments[col.source_id() as usize]
            .seq_quals
            .iter_seq_mut()
            .zip(col.iter_row_regions())
        {
            f(seq, &row_regions);
        }
    }

    /// # Panics
    /// when the tag is missing or not a String column
    pub fn apply_mut_with_string_tag<F>(&mut self, label: &TagLabel, mut f: F)
    where
        F: for<'a> FnMut(&mut SmallVec<[&BStr; 4]>, Option<&BStr>),
    {
        let TagColumn::String(col) = self.tags.get(label).expect("Tag must be present, bug") else {
            panic!("Tag {label:?} is not a String column");
        };
        for (ii, molecule) in blocks::molecules(&self.segments).enumerate() {
            let mut reads: SmallVec<[&BStr; 4]> =
                molecule.into_iter().map(|read| read.seq).collect();
            f(&mut reads, col.get_string(ii));
        }
    }

    /// # Panics
    /// when the tag is missing or not a Numeric column
    pub fn apply_mut_with_numeric_tag<F>(&mut self, label: &TagLabel, mut f: F)
    where
        F: for<'a> FnMut(&mut SmallVec<[&BStr; 4]>, f64),
    {
        let TagColumn::Numeric(tags) = self.tags.get(label).expect("Tag must be present, bug")
        else {
            panic!("Tag {label:?} is not a Numeric column");
        };
        for (ii, molecule) in blocks::molecules(&self.segments).enumerate() {
            let mut reads: SmallVec<[&BStr; 4]> =
                molecule.into_iter().map(|read| read.seq).collect();
            f(&mut reads, tags[ii]);
        }
    }

    /// # Panics
    /// when the tag is missing or not a Bool column
    pub fn apply_mut_with_bool_tag<F>(&mut self, label: &TagLabel, mut f: F)
    where
        F: for<'a> FnMut(&mut SmallVec<[&BStr; 4]>, bool),
    {
        let TagColumn::Bool(tags) = self.tags.get(label).expect("Tag must be present, bug") else {
            panic!("Tag {label:?} is not a Bool column");
        };
        for (ii, molecule) in blocks::molecules(&self.segments).enumerate() {
            let mut reads: SmallVec<[&BStr; 4]> =
                molecule.into_iter().map(|read| read.seq).collect();
            f(&mut reads, tags[ii]);
        }
    }

    // /// # Panics
    // /// when the tag is missing
    // pub fn apply_mut_with_two_tags<F>(&mut self, label: &TagLabel, other_label: &TagLabel, mut f: F)
    // where
    //     F: for<'a> FnMut(&mut [WrappedFastQReadMut<'a>], &TagValue, &TagValue),
    // {
    //     let tags = self.tags.get(label).expect("Tag must be present, bug");
    //
    //     let other_tags = self
    //         .tags
    //         .get(other_label)
    //         .expect("Tag must be present, bug");
    //
    //     for ii in 0..self.segments[0].entries.len() {
    //         let mut reads: Vec<WrappedFastQReadMut> = Vec::new();
    //         for v in &mut self.segments {
    //             reads.push(WrappedFastQReadMut(&mut v.entries[ii], &mut v.block));
    //         }
    //         f(&mut reads, &tags[ii], &other_tags[ii]);
    //         reads.clear();
    //     }
    // }

    /// # Panics
    /// When the segments have different read counts
    /// (that's the point of this function)
    pub fn sanity_check(&self) -> Result<()> {
        // The PodStack should prevent this for the segments,
        // but let's just be safe.
        let mut count = None;
        for (ii, v) in self.segments.iter().enumerate() {
            if let Some(c) = count {
                if c != v.len() {
                    bail!(
                        "Segment counts differ (unequal number of reads), expected {c}, got {} in segment {ii}",
                        v.len()
                    );
                }
            } else {
                count = Some(v.len());
            }
        }
        if let Some(count) = count
            && let Some(output_tags) = &self.output_tags
        {
            assert_eq!(
                count,
                output_tags.len(),
                "Output tag count differs, expected {count}, got {}",
                output_tags.len()
            );
        }
        Ok(())
    }

    /// Apply a function in place to all read sequencs in a segment
    pub fn apply_in_place(
        &mut self,
        segment: SegmentIndex,
        mut f: impl FnMut(&mut BStr),
        condition: Option<&[bool]>,
    ) {
        for (idx, seq) in self.segments[segment.as_index()]
            .seq_quals
            .iter_seq_mut()
            .enumerate()
        {
            if condition.is_none_or(|c| c[idx]) {
                f(seq);
            }
        }
    }
    /// Apply a function in place to all reads in a segment,
    /// allowing mutation on the reads (non length changing!)
    /// with optional condition filter
    /// wrapped
    /// for easy access.
    pub fn apply_in_place_wrapped(
        &mut self,
        segment: SegmentIndex,
        mut f: impl FnMut(&mut FastQReadMut),
        condition: Option<&[bool]>,
    ) {
        for (idx, mut read) in self.segments[segment.as_index()].iter_mut().enumerate() {
            if condition.is_none_or(|c| c[idx]) {
                f(&mut read);
            }
        }
    }

    /// `apply_in_place_wrapped`, but support `SegmentIndexOrAll::All`
    /// by iterating the function over all segmetns
    pub fn apply_in_place_wrapped_plus_all(
        &mut self,
        segment: SegmentIndexOrAll,
        mut f: impl FnMut(&mut FastQReadMut),
        condition: Option<&[bool]>,
    ) {
        for segment in self.iter_matching_segments_mut(segment) {
            for (idx, mut read) in segment.iter_mut().enumerate() {
                if condition.is_none_or(|c| c[idx]) {
                    f(&mut read);
                }
            }
        }
    }

    /* fn apply_filter(
        segment: &SegmentIndex,
        block: &mut io::FastQBlocksCombined,
        f: impl FnMut(&mut io::WrappedFastQRead) -> bool,
    ) {
        let segment_block = &block.segments[segment.get_index()];
        let keep: Vec<_> = segment_block.apply(f);
        apply_bool_filter(block, &keep);
    } */

    /// Apply a boolean filter (vec) to all segments and tags
    /// # Panics
    /// when the tag is missing
    pub fn apply_bool_filter(&mut self, keep: &[bool]) {
        let should: usize = keep.iter().map(|x| usize::from(*x)).sum();
        // Fan the filter out to every segment at once, keeping them in lockstep.
        for segment in &mut self.segments {
            segment.retain_by_bools(keep);
        }
        assert_eq!(self.row_count(), should);
        for tag_entries in self.tags.values_mut() {
            tag_entries.retain_by_bool(keep);
            assert_eq!(tag_entries.len(), should);
        }
        if let Some(output_tags) = self.output_tags.as_mut() {
            let mut iter = keep.iter();
            output_tags.retain(|_| {
                *iter
                    .next()
                    .expect("iterator has exact number of elements matching filter")
            });
            assert_eq!(output_tags.len(), should);
        }
    }

    /// Create a  location tag builder referencing the specific `FastQChunk`
    #[must_use]
    #[expect(clippy::cast_possible_truncation, reason = "Segments always <= 255")]
    pub fn location_column_builder(
        &self,
        segment: SegmentIndex,
    ) -> stringpod::DualStringPodMultiLocationAliasBuilder<'_> {
        // Stamp the source segment so the resulting Location column remembers
        // which segment it was aliased from (recovered via
        // `TagColumn::location_segment`). Needed by steps that later rebuild a
        // location column against the same segment (e.g. ReservoirSample).
        self.segments[segment.as_index()]
            .seq_quals
            .multi_location_alias_builder()
            .with_source_id(segment.as_index() as u32)
    }

    /// Create a  location tag builder referencing the specific `FastQChunk`
    /// and return references to other parts, spliting the struct temporarily
    #[expect(clippy::cast_possible_truncation, reason = "Segments always <= 255")]
    pub fn location_column_builder_and_tags_and_segment(
        &mut self,
        segment_index: SegmentIndex,
    ) -> (
        stringpod::DualStringPodMultiLocationAliasBuilder<'_>,
        &mut IndexMap<TagLabel, TagColumn>,
        &FastQChunk,
    ) {
        // Stamp the source segment so the resulting Location column remembers
        // which segment it was aliased from (recovered via
        // `TagColumn::location_segment`). Needed by steps that later rebuild a
        // location column against the same segment (e.g. ReservoirSample).
        (
            self.segments[segment_index.as_index()]
                .seq_quals
                .multi_location_alias_builder()
                .with_source_id(segment_index.as_index() as u32),
            &mut self.tags,
            &self.segments[segment_index.as_index()],
        )
    }
}

/// Mutable handle to one [`FastQBlocksCombined`] segment, from
/// [`FastQBlocksCombined::member_mut`]. Derefs to the segment; re-validates the
/// segment's read count on drop so a count change can't silently break the
/// lockstep invariant.
pub struct MemberGuard<'a> {
    segment: &'a mut FastQChunk,
    expected: usize,
}

impl std::ops::Deref for MemberGuard<'_> {
    type Target = FastQChunk;
    fn deref(&self) -> &FastQChunk {
        self.segment
    }
}

impl std::ops::DerefMut for MemberGuard<'_> {
    fn deref_mut(&mut self) -> &mut FastQChunk {
        self.segment
    }
}

impl Drop for MemberGuard<'_> {
    fn drop(&mut self) {
        // Don't mask an in-flight panic with a second one (that would abort).
        if std::thread::panicking() {
            return;
        }
        let now = self.segment.row_count();
        assert_eq!(
            now, self.expected,
            "FastQBlocksCombined::member_mut changed a segment's read count from {} to {now}: \
             use truncate/drain/apply_bool_filter so every segment stays in lockstep",
            self.expected,
        );
    }
}

#[must_use]
pub fn longest_suffix_that_is_a_prefix(
    seq: &[u8],
    query: &[u8],
    max_mismatches: usize,
    min_length: NonZero<usize>,
) -> Option<usize> {
    let max_len = std::cmp::min(seq.len(), query.len());
    for prefix_len in (min_length.into()..=max_len).rev() {
        let suffix_start = seq.len() - prefix_len;
        let dist = hamming(&seq[suffix_start..], &query[..prefix_len]) as usize;
        if dist <= max_mismatches {
            return Some(prefix_len);
        }
    }
    None
}
