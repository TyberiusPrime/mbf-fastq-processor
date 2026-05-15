use anyhow::{Result, bail};
use bstr::{BStr, BString, ByteVec};
use indexmap::IndexMap;
use std::marker::PhantomData;
use std::num::NonZero;
use std::ops::Range;

use fastqrab_config::{TagLabel, segments::SegmentIndexOrAll};
use fastqrab_dna::dna::{
    Anchor, HitRegion, Hits, TagColumn, TagValue, find_iupac, find_iupac_with_indel, hamming,
    reverse_complement_iupac,
};
use fastqrab_dna::segments::SegmentIndex;

pub type DemultiplexTag = u64;
pub type Tags = IndexMap<TagLabel, Vec<TagValue>>;
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

#[derive(Debug)]
pub enum FastQElement {
    Owned(Vec<u8>),
    Local(Position),
}

impl FastQElement {
    // #[must_use]
    // pub fn to_owned(&self, block: &[u8]) -> Self {
    //     Self::Owned(self.get(block).to_vec())
    // }

    #[must_use]
    pub fn get<'a>(&'a self, block: &'a [u8]) -> &'a [u8] {
        match self {
            FastQElement::Owned(v) => &v[..],
            FastQElement::Local(p) => &block[p.start..p.end],
        }
    }

    #[must_use]
    pub fn get_mut<'a>(&'a mut self, block: &'a mut [u8]) -> &'a mut [u8] {
        match self {
            FastQElement::Owned(v) => &mut v[..],
            FastQElement::Local(p) => &mut block[p.start..p.end],
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            FastQElement::Owned(v) => v.len(),
            FastQElement::Local(p) => p.end - p.start,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            FastQElement::Owned(v) => v.is_empty(),
            FastQElement::Local(p) => p.start == p.end,
        }
    }

    /// # Panics
    /// When the len exceeds usize
    pub fn replace<'a>(&'a mut self, new_value: &[u8], block: &'a mut Vec<u8>) {
        match self {
            FastQElement::Owned(_) => {
                *self = FastQElement::Owned(new_value.to_vec());
            }
            FastQElement::Local(inner) => {
                if inner.end - inner.start >= new_value.len() {
                    inner.end = inner
                        .start
                        .checked_add(new_value.len())
                        .expect("new length exceeded usize");
                    block[inner.start..inner.end].copy_from_slice(new_value);
                } else {
                    let new_start = block.len();
                    let new_total_len = new_start
                        .checked_add(new_value.len())
                        .expect("New read size exceeds usize");
                    // Resize buffer to accommodate old data + new text
                    block.resize(new_total_len, 0);
                    //copy in the new text
                    block[new_start..new_total_len].copy_from_slice(new_value);

                    inner.start = new_start;
                    inner.end = new_total_len;
                }
            }
        }
    }

    /// Cut the first n bases
    fn cut_start(&mut self, n: usize) {
        match self {
            FastQElement::Owned(element) => {
                element.drain(0..n.min(element.len()));
            }
            FastQElement::Local(element) => {
                let new_end = (element.start + n).min(element.end);
                element.start = new_end;
                //assert!(element.start <= element.end);
            }
        }
    }

    fn cut_end(&mut self, n: usize) {
        match self {
            FastQElement::Owned(element) => element.resize(element.len().saturating_sub(n), 0),
            FastQElement::Local(element) => {
                let new_end = element.end.saturating_sub(n).max(element.start);
                element.end = new_end;
            }
        }
    }

    fn prefix(&mut self, text: &[u8], block: &mut Vec<u8>) {
        match self {
            FastQElement::Owned(inner) => {
                let mut new = Vec::with_capacity(inner.len() + text.len());
                new.extend(text);
                new.extend(&inner[..]);
                *self = FastQElement::Owned(new);
            }
            FastQElement::Local(inner) => {
                //we allocate these into the existing large block
                //this two major advantages when every read get's modified
                //since we safe a ton of separate allocations (and drops!)
                let old_len = inner.end - inner.start;
                let new_start = block.len();
                let new_total_len = new_start + old_len + text.len();
                let new_split = new_start + text.len();

                // Resize buffer to accommodate old data + new text
                block.resize(new_total_len, 0);
                //copy in the new prefix
                block[new_start..new_split].copy_from_slice(text);

                // Copy old data to the end using copy_within (safe, non-overlapping)
                block.copy_within(inner.start..inner.end, new_split);

                inner.start = new_start;
                inner.end = new_total_len;
            }
        }
    }

    fn postfix(&mut self, text: &[u8], block: &mut Vec<u8>) {
        match self {
            FastQElement::Owned(inner) => inner.extend(text),
            FastQElement::Local(inner) => {
                //we allocate these into the existing large block
                //this has major advantages when every read get's modified
                //since we safe a ton of separate allocations (and drops!)
                let old_len = inner.end - inner.start;
                let new_start = block.len();
                let new_total_len = new_start + old_len + text.len();

                // Resize buffer to accommodate old data + new text
                block.resize(new_total_len, 0);

                // Copy old data to the end using copy_within (safe, non-overlapping)
                block.copy_within(inner.start..inner.end, new_start);

                // Copy new text after old data
                block[new_start + old_len..new_total_len].copy_from_slice(text);

                inner.start = new_start;
                inner.end = new_total_len;
            }
        }
    }

    fn reverse(&mut self, local_buffer: &mut [u8]) {
        self.get_mut(local_buffer).reverse();
    }

    fn reverse_complement(&mut self, local_buffer: &mut [u8]) {
        let m = self.get_mut(local_buffer);
        let reversed = reverse_complement_iupac(m);
        assert!(reversed.len() == m.len());
        m.copy_from_slice(&reversed[..m.len()]);
    }

    /// Swap two `FastQElements` without allocating new memory when possible.
    /// This handles all combinations of Owned/Local variants efficiently.
    fn swap_with(
        &mut self,
        other: &mut FastQElement,
        self_block: &mut [u8],
        other_block: &mut [u8],
    ) {
        match (&mut *self, &mut *other) {
            // Both Local: Need to swap actual data between blocks since positions
            // are only valid for their original blocks
            (FastQElement::Local(pos_self), FastQElement::Local(pos_other)) => {
                let self_data = &mut self_block[pos_self.start..pos_self.end];
                let other_data = &mut other_block[pos_other.start..pos_other.end];

                let self_copy = self_data.to_vec();

                // Try to reuse self's block space for other's data
                if other_data.len() <= self_data.len() {
                    self_data[..other_data.len()].copy_from_slice(other_data);
                    pos_self.end = pos_self.start + other_data.len();
                } else {
                    // Doesn't fit, make it owned
                    *self = FastQElement::Owned(other_data.to_vec());
                }

                // Try to reuse other's block space for self's data
                if self_copy.len() <= other_data.len() {
                    other_data[..self_copy.len()].copy_from_slice(&self_copy);
                    pos_other.end = pos_other.start + self_copy.len();
                } else {
                    // Doesn't fit, make it owned
                    *other = FastQElement::Owned(self_copy);
                }
            }
            // Both Owned: swap the Vec<u8>
            (FastQElement::Owned(vec_a), FastQElement::Owned(vec_b)) => {
                std::mem::swap(vec_a, vec_b);
            }
            // Local <- Owned: Try to reuse block space if the owned data fits
            (FastQElement::Local(pos_self), FastQElement::Owned(vec_other)) => {
                let self_data = self_block[pos_self.start..pos_self.end].to_vec();
                let self_len = pos_self.end - pos_self.start;
                let other_len = vec_other.len();

                if other_len <= self_len {
                    // The owned data fits in our local block space - reuse it
                    self_block[pos_self.start..pos_self.start + other_len]
                        .copy_from_slice(vec_other);
                    pos_self.end = pos_self.start + other_len;
                    // Replace other's owned vec with self's data
                    *vec_other = self_data;
                } else {
                    // The owned data doesn't fit - take ownership
                    let new_self = FastQElement::Owned(std::mem::take(vec_other));
                    *vec_other = self_data;
                    *self = new_self;
                }
            }
            // Owned <- Local: Copy local data into owned vec
            (FastQElement::Owned(vec_self), FastQElement::Local(pos_other)) => {
                let other_data = other_block[pos_other.start..pos_other.end].to_vec();
                let self_len = vec_self.len();
                let other_len = pos_other.end - pos_other.start; // mutants false positive.

                if self_len <= other_len {
                    // Our owned data fits in the other's local block space - swap using block
                    other_block[pos_other.start..pos_other.start + self_len]
                        .copy_from_slice(vec_self);
                    pos_other.end = pos_other.start + self_len;
                    // Replace our owned vec with other's data
                    *vec_self = other_data;
                } else {
                    // Our owned data doesn't fit - other needs to become owned
                    let new_other = FastQElement::Owned(std::mem::take(vec_self));
                    *vec_self = other_data;
                    *other = new_other;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct FastQRead {
    pub name: FastQElement,
    pub seq: FastQElement,
    pub qual: FastQElement,
}

impl FastQRead {
    #[track_caller]
    pub(crate) fn new(
        name: FastQElement,
        seq: FastQElement,
        qual: FastQElement,
    ) -> Result<FastQRead> {
        let res = FastQRead { name, seq, qual };
        res.verify()?;
        Ok(res)
    }

    // #[must_use]
    // pub fn to_owned(&self, block: &[u8]) -> FastQRead {
    //     FastQRead {
    //         name: self.name.to_owned(block),
    //         seq: self.seq.to_owned(block),
    //         qual: self.qual.to_owned(block),
    //     }
    // }

    #[track_caller]
    pub fn verify(&self) -> Result<()> {
        if self.seq.len() != self.qual.len() {
            bail!(
                "Sequence and quality must have the same length. Check your input fastq. Wrapped FASTQ is not supported."
            );
        }
        Ok(())
    }

    /// # Panics
    /// When lengths differ after cutting
    pub fn cut_start(&mut self, n: usize) {
        self.seq.cut_start(n);
        self.qual.cut_start(n);
        assert_eq!(self.seq.len(), self.qual.len());
    }

    /// # Panics
    /// When lengths differ after cutting
    pub fn cut_end(&mut self, n: usize) {
        self.seq.cut_end(n);
        self.qual.cut_end(n);
        assert_eq!(self.seq.len(), self.qual.len());
    }

    /// # Panics
    /// When lengths differ after cutting
    pub fn max_len(&mut self, n: usize) {
        let len = self.seq.len().min(n);
        self.seq.cut_end(self.seq.len() - len);
        self.qual.cut_end(self.qual.len() - len);
        assert_eq!(self.seq.len(), self.qual.len());
    }

    /// Swap two `FastQReads` without allocating when possible
    pub fn swap_with(
        &mut self,
        other: &mut FastQRead,
        self_block: &mut [u8],
        other_block: &mut [u8],
    ) {
        self.name
            .swap_with(&mut other.name, self_block, other_block);
        self.seq.swap_with(&mut other.seq, self_block, other_block);
        self.qual
            .swap_with(&mut other.qual, self_block, other_block);
    }
}

pub enum NewLocation {
    Remove,
    Keep,
    New(HitRegion),
    NewWithSeq(HitRegion, BString),
}

pub struct FastQBlock {
    pub block: Vec<u8>,
    pub entries: Vec<FastQRead>,
    pub first_read_sequential_number: usize,
}

// cov:excl-start
#[expect(clippy::missing_fields_in_debug, reason = "the point")]
impl std::fmt::Debug for FastQBlock {
    #[mutants::skip] // debugging only.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastQBlock")
            .field("block_len", &self.block.len())
            .field("entries_len", &self.entries.len())
            .finish()
    }
}
// cov:excl-stop

impl Clone for FastQBlock {
    ///we can clone complete `FastQblocks`, but we can't clone individual reads.
    fn clone(&self) -> Self {
        let new_block = self.block.clone();
        let new_entries = self
            .entries
            .iter()
            .map(|entry| FastQRead {
                name: match &entry.name {
                    FastQElement::Owned(items) => FastQElement::Owned(items.clone()),
                    FastQElement::Local(position) => FastQElement::Local(*position),
                },

                seq: match &entry.seq {
                    FastQElement::Owned(items) => FastQElement::Owned(items.clone()),
                    FastQElement::Local(position) => FastQElement::Local(*position),
                },

                qual: match &entry.qual {
                    FastQElement::Owned(items) => FastQElement::Owned(items.clone()),
                    FastQElement::Local(position) => FastQElement::Local(*position),
                },
            })
            .collect();
        FastQBlock {
            block: new_block,
            entries: new_entries,
            first_read_sequential_number: self.first_read_sequential_number,
        }
    }
}

impl FastQBlock {
    #[must_use]
    pub fn empty() -> FastQBlock {
        FastQBlock {
            block: Vec::new(),
            entries: Vec::new(),
            first_read_sequential_number: 0,
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> WrappedFastQRead<'_> {
        WrappedFastQRead(&self.entries[index], &self.block)
    }

    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> WrappedFastQReadMut<'_> {
        WrappedFastQReadMut(&mut self.entries[index], &mut self.block)
    }

    #[must_use]
    pub fn get_pseudo_iter(&self) -> FastQBlockPseudoIter<'_> {
        FastQBlockPseudoIter::Simple {
            pos: 0,
            inner: self,
        }
    }

    /// # Panics
    /// When read construction fails, because of unequal length afterward
    pub fn append_read(&mut self, read: &WrappedFastQRead<'_>) {
        let local_read = FastQRead::new(
            self.append_element(read.0.name.get(read.1)),
            self.append_element(read.0.seq.get(read.1)),
            self.append_element(read.0.qual.get(read.1)),
        )
        .expect("Constructing read from existing read failed?!");
        self.entries.push(local_read);
    }

    pub fn replace_read(&mut self, index: usize, read: &WrappedFastQRead<'_>) {
        let local_read = &mut self.entries[index];
        local_read
            .name
            .replace(read.0.name.get(read.1), &mut self.block);
        local_read
            .seq
            .replace(read.0.seq.get(read.1), &mut self.block);
        local_read
            .qual
            .replace(read.0.qual.get(read.1), &mut self.block);
    }

    /// Add one byte-string as a `FastQElement::Local` by extending the block
    pub fn append_element(&mut self, text: &[u8]) -> FastQElement {
        let start = self.block.len();
        let end = start + text.len();
        self.block.extend_from_slice(text);
        FastQElement::Local(Position { start, end })
    }

    /// Add a byte iterator as a `FastQElement::Local` by extending the block
    /// # Panics
    /// When expected and actual final len differ?
    pub fn append_element_from_iter<T>(&mut self, iter: T, len: usize) -> FastQElement
    where
        T: Iterator<Item = u8>,
    {
        let start = self.block.len();
        let end = start + len;
        self.block.extend(iter.take(len));
        let new_len = self.block.len();
        assert_eq!(
            end, new_len,
            "Appended length does not match expected length. Wrong len for iter?"
        );
        FastQElement::Local(Position { start, end })
    }

    #[must_use]
    pub fn get_pseudo_iter_including_tag<'a>(
        &'a self,
        output_tags: &'a Option<Vec<DemultiplexTag>>,
    ) -> FastQBlockPseudoIterIncludingTag<'a> {
        FastQBlockPseudoIterIncludingTag {
            pos: 0,
            inner: self,
            output_tags,
        }
    }

    #[must_use]
    pub fn get_pseudo_iter_filtered_to_tag<'a>(
        &'a self,
        tag: DemultiplexTag,
        output_tags: &'a Vec<DemultiplexTag>,
    ) -> FastQBlockPseudoIter<'a> {
        FastQBlockPseudoIter::Filtered {
            pos: 0,
            inner: self,
            tag,
            output_tags,
        }
    }

    pub fn apply<T>(&self, mut f: impl FnMut(&mut WrappedFastQRead) -> T) -> Vec<T> {
        let mut res = Vec::new();
        for entry in &self.entries {
            let mut wrapped = WrappedFastQRead(entry, &self.block);
            res.push(f(&mut wrapped));
        }
        res
    }

    pub fn apply_mut(&mut self, mut f: impl FnMut(&mut WrappedFastQReadMut)) {
        for entry in &mut self.entries {
            let mut wrapped = WrappedFastQReadMut(entry, &mut self.block);
            f(&mut wrapped);
        }
    }

    /// # Panics
    /// when conditions & tag column have different lengths
    pub fn apply_mut_conditional(
        &mut self,
        mut f: impl FnMut(&mut WrappedFastQReadMut),
        condition: &[bool],
    ) {
        assert_eq!(
            condition.len(),
            self.entries.len(),
            "Condition and entries must have the same length"
        );
        for (idx, entry) in self.entries.iter_mut().enumerate() {
            if condition[idx] {
                let mut wrapped = WrappedFastQReadMut(entry, &mut self.block);
                f(&mut wrapped);
            }
        }
    }

    pub fn apply_with_demultiplex_tag<T>(
        &self,
        mut f: impl FnMut(&mut WrappedFastQRead, DemultiplexTag) -> T,
        output_tags: Option<&Vec<DemultiplexTag>>,
    ) -> Vec<T> {
        let mut res = Vec::new();
        for (pos, entry) in self.entries.iter().enumerate() {
            let output_tag = output_tags.map(|x| x[pos]).unwrap_or_default();
            let mut wrapped = WrappedFastQRead(entry, &self.block);
            res.push(f(&mut wrapped, output_tag));
        }
        res
    }

    // pub fn apply_mut_with_tag(
    //     &mut self,
    //     tags: &IndexMap<TagLabel, Vec<TagValue>>,
    //     label: &TagLabel,
    //     f: impl Fn(&mut WrappedFastQReadMut, &TagValue),
    // ) {
    //     let tags = tags
    //         .get(label)
    //         .expect("Tag not set, should have been caught earlier");
    //     assert_eq!(
    //         tags.len(),
    //         self.entries.len(),
    //         "Tags and entries must have the same length. Label: {label} ",
    //     );
    //     for (ii, entry) in &mut self.entries.iter_mut().enumerate() {
    //         let mut wrapped = WrappedFastQReadMut(entry, &mut self.block);
    //         f(&mut wrapped, &tags[ii]);
    //     }
    // }

    /// # Panics
    /// When run with less than two entries? Don't think that can happen
    #[must_use]
    pub fn split_at(mut self, target_reads_per_block: NonZero<usize>) -> (FastQBlock, FastQBlock) {
        if self.len() <= target_reads_per_block.into() {
            (self, FastQBlock::empty())
        } else {
            let target_reads_per_block: usize = target_reads_per_block.into();
            let mut right: Vec<FastQRead> = self.entries.drain(target_reads_per_block..).collect();
            let left = self.entries;
            //let (left, right) = self.entries.split_at(target_reads_per_block);
            let buffer_split_pos = match &left
                .iter()
                .last()
                .expect("left buffer must have at least one element")
                .qual
            {
                FastQElement::Owned(_) => match &right
                    .first()
                    .expect("right buffer must have at least one element")
                    .name
                {
                    // cov:excl-start
                    FastQElement::Owned(_) => {
                        unreachable!("Left and write were owned, that shouldn't happen")
                    }
                    // cov:excl-stop
                    FastQElement::Local(position) => position.start,
                },
                FastQElement::Local(position) => position.end,
            };
            for entry in &mut right {
                match &mut entry.name {
                    // cov:excl-start
                    FastQElement::Owned(_) => {
                        unreachable!()
                    }
                    // cov:excl-stop
                    FastQElement::Local(position) => {
                        position.start -= buffer_split_pos;
                        position.end -= buffer_split_pos;
                    }
                }
                match &mut entry.seq {
                    // cov:excl-start
                    FastQElement::Owned(_) => {
                        unreachable!()
                    }
                    // cov:excl-stop
                    FastQElement::Local(position) => {
                        position.start -= buffer_split_pos;
                        position.end -= buffer_split_pos;
                    }
                }
                match &mut entry.qual {
                    // cov:excl-start
                    FastQElement::Owned(_) => {
                        unreachable!()
                    }
                    // cov:excl-stop
                    FastQElement::Local(position) => {
                        position.start -= buffer_split_pos;
                        position.end -= buffer_split_pos;
                    }
                }
            }
            let right_buf = self.block.drain(buffer_split_pos..).collect();
            let left_buf = self.block;
            let count_left = left.len();
            (
                FastQBlock {
                    block: left_buf,
                    entries: left,
                    first_read_sequential_number: self.first_read_sequential_number,
                },
                FastQBlock {
                    block: right_buf,
                    entries: right,
                    first_read_sequential_number: self.first_read_sequential_number + count_left,
                },
            )
        }
    }

    #[must_use]
    pub fn split_interleaved(self, interleave_count: NonZero<usize>) -> Vec<FastQBlock> {
        let mut outputs = Vec::new();
        for _ in 0..interleave_count.into() {
            outputs.push(FastQBlock {
                block: self.block.clone(),
                entries: Vec::new(),
                first_read_sequential_number: self.first_read_sequential_number,
            });
        }
        for (ii, entry) in self.entries.into_iter().enumerate() {
            outputs[ii % interleave_count].entries.push(entry);
        }
        outputs
    }
}

pub enum FastQBlockPseudoIter<'a> {
    Simple {
        pos: usize,
        inner: &'a FastQBlock,
    },
    Filtered {
        pos: usize,
        inner: &'a FastQBlock,
        tag: DemultiplexTag,
        output_tags: &'a Vec<DemultiplexTag>,
    },
}

impl<'a> FastQBlockPseudoIter<'a> {
    pub fn pseudo_next(&mut self) -> Option<WrappedFastQRead<'a>> {
        match self {
            FastQBlockPseudoIter::Simple { pos, inner } => {
                let len = inner.entries.len();
                if *pos >= len || len == 0 {
                    return None;
                }
                let e = WrappedFastQRead(&inner.entries[*pos], &inner.block);
                *pos += 1;
                Some(e)
            }
            FastQBlockPseudoIter::Filtered {
                pos,
                inner,
                tag,
                output_tags,
            } => {
                let len = inner.entries.len();
                loop {
                    if *pos >= len || len == 0 {
                        return None;
                    }
                    if output_tags[*pos] == *tag {
                        let e = WrappedFastQRead(&inner.entries[*pos], &inner.block);
                        *pos += 1;
                        return Some(e);
                    } else {
                        *pos += 1;
                    }
                }
            }
        }
    }

    /// Like `pseudo_next` but also returns the original block index of the read.
    /// This is needed for looking up per-read tags when iterating a filtered view.
    pub fn pseudo_next_with_index(&mut self) -> Option<(WrappedFastQRead<'a>, usize)> {
        match self {
            FastQBlockPseudoIter::Simple { pos, inner } => {
                let len = inner.entries.len();
                if *pos >= len || len == 0 {
                    return None;
                }
                let idx = *pos;
                let e = WrappedFastQRead(&inner.entries[idx], &inner.block);
                *pos += 1;
                Some((e, idx))
            }
            FastQBlockPseudoIter::Filtered {
                pos,
                inner,
                tag,
                output_tags,
            } => {
                let len = inner.entries.len();
                loop {
                    if *pos >= len || len == 0 {
                        return None;
                    }
                    let idx = *pos;
                    if output_tags[idx] == *tag {
                        let e = WrappedFastQRead(&inner.entries[idx], &inner.block);
                        *pos += 1;
                        return Some((e, idx));
                    } else {
                        *pos += 1;
                    }
                }
            }
        }
    }
}

pub struct FastQBlockPseudoIterIncludingTag<'a> {
    pos: usize,
    inner: &'a FastQBlock,
    output_tags: &'a Option<Vec<DemultiplexTag>>,
}

impl<'a> FastQBlockPseudoIterIncludingTag<'a> {
    pub fn pseudo_next(&mut self) -> Option<(WrappedFastQRead<'a>, DemultiplexTag)> {
        let pos = &mut self.pos;
        let len = self.inner.entries.len();
        if *pos >= len || len == 0 {
            return None;
        }
        let e = (
            WrappedFastQRead(&self.inner.entries[*pos], &self.inner.block),
            match &self.output_tags {
                Some(tags) => tags[*pos],
                None => 0,
            },
        );
        *pos += 1;
        Some(e)
    }
}

pub struct WrappedFastQRead<'a>(&'a FastQRead, &'a Vec<u8>);
pub struct WrappedFastQReadMut<'a>(&'a mut FastQRead, &'a mut Vec<u8>);

// cov:excl-start
impl std::fmt::Debug for WrappedFastQRead<'_> {
    #[mutants::skip] // debugging only.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = BStr::new(self.name());
        let seq = BStr::new(self.seq());
        f.write_str(&format!("WrappedFastQRead {{ name: {name}, seq: {seq} }}",))
    }
}
// cov:excl-stop

// cov:excl-start
impl std::fmt::Debug for WrappedFastQReadMut<'_> {
    #[mutants::skip] // debugging only.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = std::str::from_utf8(self.name()).expect("FASTQ field should be valid UTF-8");
        let seq = std::str::from_utf8(self.seq()).expect("FASTQ field should be valid UTF-8");
        f.write_str(&format!(
            "WrappedFastQReadMut {{ name: {name}, seq: {seq} }}",
        ))
    }
}
// cov:excl-stop

pub trait WrappedFastQReadCommon {
    #[must_use]
    fn name(&self) -> &[u8];

    #[must_use]
    fn seq(&self) -> &[u8];
    #[must_use]
    fn qual(&self) -> &[u8];

    //get only the name part (up to the first read_comment_insert_char, or in full
    //if note is present)
    #[must_use]
    fn name_without_comment(&self, read_comment_insert_char: u8) -> &[u8] {
        let full = self.name();
        let pos_of_first_space = full.iter().position(|&x| x == read_comment_insert_char);
        match pos_of_first_space {
            Some(pos) => &full[..pos],
            None => full,
        }
    }

    /// get only the comment (without `read_comment_insert_char`)
    /// or None if not present
    #[must_use]
    fn name_only_comment(&self, read_comment_insert_char: u8) -> Option<&[u8]> {
        //read comment character to a top level input (i suppose) and have them use this
        let full = self.name();
        let pos_of_first_space = full.iter().position(|&x| x == read_comment_insert_char);
        match pos_of_first_space {
            Some(pos) => Some(&full[pos + 1..]),
            None => None,
        }
    }

    #[must_use]
    fn len(&self) -> usize {
        self.seq().len()
    }

    #[must_use]
    fn is_empty(&self) -> bool {
        self.seq().is_empty()
    }

    fn append_as_fastq(&self, out: &mut Vec<u8>) {
        let name = self.name();
        let seq = self.seq();
        let qual = self.qual();
        out.push(b'@');

        out.extend(name);

        out.push(b'\n');
        out.extend(seq);

        out.extend(b"\n+\n");
        out.extend(qual);
        out.push(b'\n');
    }

    fn as_fasta(&self, out: &mut Vec<u8>) {
        let name = self.name();
        let seq = self.seq();
        out.push(b'>');
        out.extend(name);
        out.push(b'\n');
        out.extend(seq);
        out.push(b'\n');
    }

    #[must_use]
    fn find_iupac(
        &self,
        query: &[u8],
        anchor: Anchor,
        max_mismatches: u8,
        target: SegmentIndex,
        max_anchor_distance: usize,
    ) -> Option<Hits> {
        let seq = self.seq();
        find_iupac(
            seq,
            query,
            anchor,
            max_mismatches,
            target,
            max_anchor_distance,
        )
    }

    #[must_use]
    fn find_iupac_with_indel(
        &self,
        query: &[u8],
        anchor: Anchor,
        max_mismatches: usize,
        max_indel_bases: usize,
        max_total_edits: Option<usize>,
        target: SegmentIndex,
    ) -> Option<Hits> {
        let seq = self.seq();
        find_iupac_with_indel(
            seq,
            query,
            anchor,
            max_mismatches,
            max_indel_bases,
            max_total_edits,
            target,
        )
    }

    // fn to_owned(&self) -> FastQRead {
    //     FastQRead {
    //         name: FastQElement::Owned(self.name().to_vec()),
    //         seq: FastQElement::Owned(self.seq().to_vec()),
    //         qual: FastQElement::Owned(self.qual().to_vec()),
    //     }
    // }
}

impl WrappedFastQReadCommon for WrappedFastQRead<'_> {
    fn name(&self) -> &[u8] {
        self.0.name.get(self.1)
    }
    fn seq(&self) -> &[u8] {
        self.0.seq.get(self.1)
    }
    fn qual(&self) -> &[u8] {
        self.0.qual.get(self.1)
    }
}

impl WrappedFastQReadCommon for WrappedFastQReadMut<'_> {
    fn name(&self) -> &[u8] {
        self.0.name.get(self.1)
    }
    fn seq(&self) -> &[u8] {
        self.0.seq.get(self.1)
    }
    fn qual(&self) -> &[u8] {
        self.0.qual.get(self.1)
    }
}

impl WrappedFastQReadMut<'_> {
    #[must_use]
    pub fn seq_mut(&mut self) -> &mut [u8] {
        self.0.seq.get_mut(self.1)
    }

    pub fn cut_start(&mut self, n: usize) {
        self.0.cut_start(n);
    }

    pub fn max_len(&mut self, n: usize) {
        self.0.max_len(n);
    }

    /// # Panics
    /// when the len invariant is violated
    pub fn prefix(&mut self, seq: &[u8], qual: &[u8]) {
        self.0.seq.prefix(seq, self.1);
        self.0.qual.prefix(qual, self.1);
        assert_eq!(self.0.seq.len(), self.0.qual.len());
    }

    /// # Panics
    /// when the len invariant is violated
    pub fn postfix(&mut self, seq: &[u8], qual: &[u8]) {
        self.0.seq.postfix(seq, self.1);
        self.0.qual.postfix(qual, self.1);
        assert_eq!(self.0.seq.len(), self.0.qual.len());
    }

    pub fn reverse_complement(&mut self) {
        self.0.seq.reverse_complement(self.1);
        self.0.qual.reverse(self.1);
    }

    /// # Panics
    /// when the len invariant is violated
    pub fn replace_seq(&mut self, new_seq: &[u8], new_qual: &[u8]) {
        assert!(new_seq.len() == new_qual.len());
        self.0.seq.replace(new_seq, self.1);
        self.0.qual.replace(new_qual, self.1);
    }

    /// # Panics
    /// when the len invariant is violated
    pub fn replace_seq_keep_qual(&mut self, new_seq: &[u8]) {
        assert!(new_seq.len() == self.0.qual.len());
        self.0.seq.replace(new_seq, self.1);
    }

    pub fn replace_name(&mut self, new_name: &[u8]) {
        self.0.name.replace(new_name, self.1);
    }

    pub fn replace_qual(&mut self, new_qual: &[u8]) {
        self.0.qual.replace(new_qual, self.1);
    }

    /// Clear both sequence and quality, leaving them empty
    pub fn clear(&mut self) {
        self.replace_seq(&[], &[]);
    }

    // pub fn trim_adapter_mismatch_tail(
    //     &mut self,
    //     query: &[u8],
    //     min_length: usize,
    //     max_mismatches: usize,
    // ) {
    //     let seq = self.seq();
    //     if query.len() > seq.len() {
    //         return;
    //     }
    //
    //     if let Some(suffix_len) =
    //         longest_suffix_that_is_a_prefix(seq, query, max_mismatches, min_length)
    //     {
    //         panic!("{suffix_len}");
    //         let should = &seq[..seq.len() - suffix_len].to_vec();
    //         self.0.seq.cut_end(suffix_len);
    //         assert_eq!(self.seq(), should);
    //         self.0.qual.cut_end(suffix_len);
    //     }
    // }

    /// # Panics
    /// when the len invariant is violated
    #[expect(clippy::too_many_lines, reason = "we need them")]
    #[expect(clippy::cast_precision_loss, reason = "acceptable")]
    pub fn trim_poly_base_suffix(
        &mut self,
        min_length: usize,
        max_mismatch_fraction: f32,
        max_consecutive_mismatches: usize,
        base: u8,
    ) {
        fn calc_run_length(
            seq: &[u8],
            query: u8,
            min_length: usize,
            max_mismatch_fraction: f32,
            max_consecutive_mismatches: usize,
        ) -> Option<usize> {
            if seq.len() < min_length {
                // optimization
                return None;
            }
            //algorithm is simple.
            // for any suffix,
            // update mismatch rate
            // if it's a match, and the mismatch rate is below the threshold,
            // and it's above the min length
            // keep the position
            // else
            // abort once even 100% matches in the remaining bases can't
            // fulfill the mismatch rate anymore.
            // or you have seen max_consecutive_mismatches
            // if no position fulfills the above, return None
            let mut matches = 0;
            let mut mismatches = 0;
            let mut last_base_pos = None;
            let seq_len = seq.len() as f32;
            let mut consecutive_mismatch_counter = 0;
            for (ii, base) in seq.iter().enumerate().rev() {
                // dbg!(
                //     ii,
                //     base,
                //     *base == query,
                //     matches, mismatches,
                //     seq_len,
                //     mismatches as f32 / (matches + mismatches) as f32,
                //     (mismatches + 1) as f32 / seq_len,
                //      consecutive_mismatch_counter,
                //      max_consecutive_mismatches,
                // );

                if *base == query {
                    matches += 1;
                    consecutive_mismatch_counter = 0;
                    if seq.len() - ii >= min_length
                        && mismatches as f32 / (matches + mismatches) as f32
                            <= max_mismatch_fraction
                    {
                        last_base_pos = Some(ii);
                    }
                } else {
                    mismatches += 1;
                    if mismatches as f32 / seq_len > max_mismatch_fraction {
                        //dbg!("do break - mismatch rate");
                        break;
                    }
                    consecutive_mismatch_counter += 1;
                    if consecutive_mismatch_counter >= max_consecutive_mismatches {
                        //dbg!("do break - consecutive mismatches");
                        break;
                    }
                }
            }
            last_base_pos
            //
        }
        let seq = self.seq();
        //dbg!(std::str::from_utf8(self.name()).unwrap());

        let last_pos = if base == b'.' {
            let lp_a = calc_run_length(
                seq,
                b'A',
                min_length,
                max_mismatch_fraction,
                max_consecutive_mismatches,
            );
            let lp_c = calc_run_length(
                seq,
                b'C',
                min_length,
                max_mismatch_fraction,
                max_consecutive_mismatches,
            );
            let lp_g = calc_run_length(
                seq,
                b'G',
                min_length,
                max_mismatch_fraction,
                max_consecutive_mismatches,
            );
            let lp_t = calc_run_length(
                seq,
                b'T',
                min_length,
                max_mismatch_fraction,
                max_consecutive_mismatches,
            );
            let lp_n = calc_run_length(
                seq,
                b'N',
                min_length,
                max_mismatch_fraction,
                max_consecutive_mismatches,
            );
            //dbg!(lp_a, lp_c, lp_g, lp_t, lp_n);
            //now I need to find the right most one that is not None
            let mut lp = lp_a;
            for other in [lp_g, lp_c, lp_t, lp_n] {
                lp = match (other, lp) {
                    (None, None | Some(_)) => lp,
                    (Some(_), None) => other,
                    (Some(other_), Some(lp_)) => {
                        if other_ < lp_ {
                            other
                        } else {
                            lp
                        }
                    }
                };
            }
            lp
        } else {
            calc_run_length(
                seq,
                base,
                min_length,
                max_mismatch_fraction,
                max_consecutive_mismatches,
            )
        };
        //dbg!(last_pos);
        if let Some(last_pos) = last_pos {
            let from_end = seq.len() - last_pos;
            self.0.seq.cut_end(from_end);
            self.0.qual.cut_end(from_end);
            assert!(self.0.seq.len() == self.0.qual.len());
        }
    }

    // pub fn trim_quality_start(&mut self, min_qual: u8) {
    //     let mut cut_pos = 0;
    //     let qual = self.qual();
    //     for (ii, q) in qual.iter().enumerate() {
    //         if *q < min_qual {
    //             cut_pos = ii + 1;
    //         } else {
    //             break;
    //         }
    //     }
    //     if cut_pos > 0 {
    //         self.0.seq.cut_start(cut_pos);
    //         self.0.qual.cut_start(cut_pos);
    //     }
    // }

    // pub fn trim_quality_end(&mut self, min_qual: u8) {
    //     let qual = self.qual();
    //     let mut cut_pos = qual.len();
    //     for q in qual.iter().rev() {
    //         if *q < min_qual {
    //             cut_pos -= 1;
    //         } else {
    //             break;
    //         }
    //     }
    //     let ql = qual.len();
    //     if cut_pos < qual.len() {
    //         self.0.seq.cut_end(ql - cut_pos);
    //         self.0.qual.cut_end(ql - cut_pos);
    //     }
    // }
}

pub struct SegmentsCombined<T> {
    pub segments: Vec<T>,
}

/// Multiple fastqblocks together with their tag annotation
/// and output destination.
///
/// Contract: segments must not be empty.
#[derive(Clone, Debug)]
pub struct FastQBlocksCombined {
    pub segments: Vec<FastQBlock>,
    pub output_tags: Option<Vec<DemultiplexTag>>, // used by Demultiplex
    pub tags: IndexMap<TagLabel, TagColumn>,
    pub is_final: bool,
    block_no: usize,
    _force_private: PhantomData<u8>,
}

impl FastQBlocksCombined {
    /// # Panics
    ///  if segments is empty
    #[must_use]
    pub fn new(
        segments: Vec<FastQBlock>,
        output_tags: Option<Vec<DemultiplexTag>>,
        tags: IndexMap<TagLabel, TagColumn>,
        is_final: bool,
        block_no: usize,
    ) -> Self {
        assert!(
            !segments.is_empty(),
            "Empty segments not supported in FastQBlocksCombined"
        );
        FastQBlocksCombined {
            segments,
            output_tags,
            tags,
            is_final,
            block_no,
            _force_private: PhantomData,
        }
    }

    #[must_use]
    pub fn block_no(&self) -> usize {
        self.block_no
    }

    #[must_use]
    pub fn iter_segment_indices(&self, idx: SegmentIndexOrAll) -> Vec<usize> {
        match idx {
            SegmentIndexOrAll::All => (0..self.segments.len()).collect(),
            SegmentIndexOrAll::Indexed(idx) => vec![idx],
        }
    }

    /// create an empty one with the same options filled, and same `block_no`
    #[must_use]
    pub fn empty(&self) -> FastQBlocksCombined {
        FastQBlocksCombined {
            segments: vec![FastQBlock::empty(); self.segments.len()],
            output_tags: if self.output_tags.is_some() {
                Some(Vec::new())
            } else {
                None
            },
            tags: IndexMap::default(),
            is_final: self.is_final,
            block_no: self.block_no,
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
    pub fn get_pseudo_iter(&self) -> FastQBlocksCombinedIterator<'_> {
        FastQBlocksCombinedIterator {
            pos: 0,
            inner: self,
        }
    }

    #[must_use]
    pub fn get_pseudo_iter_including_tag(&self) -> FastQBlocksCombinedIteratorIncludingTag<'_> {
        if let Some(output_tags) = &self.output_tags {
            debug_assert!(
                output_tags.len() == self.segments[0].entries.len(),
                "Output tags must have the same length as segments"
            );
        }
        FastQBlocksCombinedIteratorIncludingTag {
            pos: 0,
            inner: self,
        }
    }
    #[must_use]
    #[expect(clippy::len_without_is_empty, reason = "We never check for empty?")]
    pub fn len(&self) -> usize {
        self.segments[0].entries.len()
    }

    // #[must_use] //todo: remove
    // pub fn is_empty(&self) -> bool {
    //     panic!();
    //     self.segments[0].entries.is_empty()
    // }

    /// # Panics
    /// when the new length is larger
    pub fn resize(&mut self, len: usize) {
        for v in &mut self.segments {
            v.entries.resize_with(len, || {
                // cov:excl-start
                panic!("Read amplification not expected. Can't resize to larger")
                // cov:excl-stop
            });
        }
        if let Some(output_tags) = &mut self.output_tags {
            // cov:excl-start
            output_tags.resize_with(len, || {
                panic!("Read amplification not expected. Can't resize to larger")
            });
            // cov:excl-stop
        }
        for tags in self.tags.values_mut() {
            // cov:excl-start
            tags.resize_with(len)
            // cov:excl-stop
        }
    }

    /// # Panics
    /// when used on demultiplexed blocks
    pub fn drain(&mut self, range: Range<usize>) {
        for v in &mut self.segments {
            v.entries.drain(range.clone());
        }
        assert!(
            self.output_tags.is_none(),
            "Drain used on a demultiplexed block. I don't think that's sensible"
        );
        // if let Some(output_tags) = &mut self.output_tags {
        //     output_tags.drain(range.clone()); // cov:excl-line currently not being used, since we
        //     // only use drain in the non-demultiplexing case, completeness and future use.
        // }
    }

    pub fn apply_mut<F>(&mut self, f: F)
    where
        F: for<'a> Fn(&mut [WrappedFastQReadMut<'a>]),
    {
        let count = self.segments[0].entries.len();
        for ii in 0..count {
            let mut reads: Vec<WrappedFastQReadMut> = Vec::new();
            for v in &mut self.segments {
                reads.push(WrappedFastQReadMut(&mut v.entries[ii], &mut v.block));
            }
            f(&mut reads);
        }
    }

    /// # Panics
    /// when the tag is missing or not a Location column
    pub fn apply_mut_with_location_tag<F>(&mut self, label: &TagLabel, mut f: F)
    where
        F: for<'a> FnMut(&mut [WrappedFastQReadMut<'a>], Option<&Hits>),
    {
        let TagColumn::Location(tags) = self.tags.get(label).expect("Tag must be present, bug")
        else {
            panic!("Tag {label:?} is not a Location column");
        };
        for ii in 0..self.segments[0].entries.len() {
            let mut reads: Vec<WrappedFastQReadMut> = Vec::new();
            for v in &mut self.segments {
                reads.push(WrappedFastQReadMut(&mut v.entries[ii], &mut v.block));
            }
            f(&mut reads, tags[ii].as_ref());
            reads.clear();
        }
    }

    /// # Panics
    /// when the tag is missing or not a String column
    pub fn apply_mut_with_string_tag<F>(&mut self, label: &TagLabel, mut f: F)
    where
        F: for<'a> FnMut(&mut [WrappedFastQReadMut<'a>], Option<&BString>),
    {
        let TagColumn::String(tags) = self.tags.get(label).expect("Tag must be present, bug")
        else {
            panic!("Tag {label:?} is not a String column");
        };
        for ii in 0..self.segments[0].entries.len() {
            let mut reads: Vec<WrappedFastQReadMut> = Vec::new();
            for v in &mut self.segments {
                reads.push(WrappedFastQReadMut(&mut v.entries[ii], &mut v.block));
            }
            f(&mut reads, tags[ii].as_ref());
            reads.clear();
        }
    }

    /// # Panics
    /// when the tag is missing or not a Numeric column
    pub fn apply_mut_with_numeric_tag<F>(&mut self, label: &TagLabel, mut f: F)
    where
        F: for<'a> FnMut(&mut [WrappedFastQReadMut<'a>], Option<f64>),
    {
        let TagColumn::Numeric(tags) = self.tags.get(label).expect("Tag must be present, bug")
        else {
            panic!("Tag {label:?} is not a Numeric column");
        };
        for ii in 0..self.segments[0].entries.len() {
            let mut reads: Vec<WrappedFastQReadMut> = Vec::new();
            for v in &mut self.segments {
                reads.push(WrappedFastQReadMut(&mut v.entries[ii], &mut v.block));
            }
            f(&mut reads, tags[ii]);
            reads.clear();
        }
    }

    /// # Panics
    /// when the tag is missing or not a Bool column
    pub fn apply_mut_with_bool_tag<F>(&mut self, label: &TagLabel, mut f: F)
    where
        F: for<'a> FnMut(&mut [WrappedFastQReadMut<'a>], Option<bool>),
    {
        let TagColumn::Bool(tags) = self.tags.get(label).expect("Tag must be present, bug") else {
            panic!("Tag {label:?} is not a Bool column");
        };
        for ii in 0..self.segments[0].entries.len() {
            let mut reads: Vec<WrappedFastQReadMut> = Vec::new();
            for v in &mut self.segments {
                reads.push(WrappedFastQReadMut(&mut v.entries[ii], &mut v.block));
            }
            f(&mut reads, tags[ii]);
            reads.clear();
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
    pub fn sanity_check(&self) -> Result<()> {
        let mut count = None;
        for (ii, v) in self.segments.iter().enumerate() {
            if let Some(c) = count {
                if c != v.entries.len() {
                    bail!(
                        "Segment counts differ (unequal number of reads), expected {c}, got {} in segment {ii}",
                        v.entries.len()
                    );
                }
            } else {
                count = Some(v.entries.len());
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

    /// Apply a function in place to all reads in a segment
    /// with optional condition filter
    /// using raw `FastQRead`
    pub fn apply_in_place(
        &mut self,
        segment: SegmentIndex,
        f: impl Fn(&mut FastQRead),
        condition: Option<&[bool]>,
    ) {
        if let Some(condition) = condition {
            for (idx, read) in self.segments[segment.get_index()]
                .entries
                .iter_mut()
                .enumerate()
            {
                if condition[idx] {
                    f(read);
                }
            }
        } else {
            for read in &mut self.segments[segment.get_index()].entries {
                f(read);
            }
        }
    }

    /// Apply a function in place to all reads in a segment,
    /// with optional condition filter
    /// wrapped
    /// for easy access.
    pub fn apply_in_place_wrapped(
        &mut self,
        segment: SegmentIndex,
        f: impl FnMut(&mut WrappedFastQReadMut),
        condition: Option<&[bool]>,
    ) {
        if let Some(condition) = condition {
            self.segments[segment.get_index()].apply_mut_conditional(f, condition);
        } else {
            self.segments[segment.get_index()].apply_mut(f);
        }
    }

    /// `apply_in_place_wrapped`, but support `SegmentIndexOrAll::All`
    /// by iterating the function over all segmetns
    pub fn apply_in_place_wrapped_plus_all(
        &mut self,
        segment: SegmentIndexOrAll,
        mut f: impl FnMut(&mut WrappedFastQReadMut),
        condition: Option<&[bool]>,
    ) {
        if let Ok(target) = segment.try_into() as Result<SegmentIndex, _> {
            self.apply_in_place_wrapped(target, f, condition);
        } else if let Some(condition) = condition {
            for read_block in &mut self.segments {
                read_block.apply_mut_conditional(&mut f, condition);
            }
        } else {
            for read_block in &mut self.segments {
                read_block.apply_mut(&mut f);
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
        for segment_block in &mut self.segments {
            let mut iter = keep.iter();
            segment_block.entries.retain(|_| {
                *iter
                    .next()
                    .expect("iterator has exact number of elements matching filter")
            });
            assert_eq!(segment_block.entries.len(), should);
        }
        for tag_entries in self.tags.values_mut() {
            let mut iter = keep.iter();
            tag_entries.retain(|| {
                *iter
                    .next()
                    .expect("iterator has exact number of elements matching filter")
            });
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

    pub fn filter_tag_locations(
        &mut self,
        segment: SegmentIndex,
        f: impl Fn(&HitRegion, usize, &BString, usize) -> NewLocation,
        condition: Option<&[bool]>,
    ) {
        let reads = &self.segments[segment.get_index()].entries;

        for values in self.tags.values_mut() {
            if let TagColumn::Location(values) = values {
                for (ii, tag_val) in values.iter_mut().enumerate() {
                    // Skip if condition is present and false for this read
                    if let Some(condition) = condition
                        && !condition[ii]
                    {
                        continue;
                    }

                    let read_length = reads[ii].seq.len();
                    if let Some(hits) = tag_val {
                        let mut any_none = false;
                        for hit in &mut hits.0 {
                            if let Some(location) = hit.location.as_mut()
                                && location.segment_index == segment
                            {
                                let sequence = &hit.sequence;
                                match f(location, ii, sequence, read_length) {
                                    NewLocation::Remove => {
                                        hit.location = None;
                                        any_none = true;
                                        break;
                                    }
                                    NewLocation::Keep => {}
                                    NewLocation::New(new) => *location = new,
                                    NewLocation::NewWithSeq(new_loc, new_seq) => {
                                        *location = new_loc;
                                        hit.sequence = new_seq;
                                    }
                                }
                            }
                        }
                        // if any are no longer present, remove all location spans
                        if any_none {
                            for hit in &mut hits.0 {
                                hit.location = None;
                            }
                        }
                    } else {
                        // no hits, so no location to change
                    }
                }
            }
        }
    }

    pub fn filter_tag_locations_beyond_read_length(&mut self, segment: SegmentIndex) {
        self.filter_tag_locations(
            segment,
            |location: &HitRegion, _pos, _seq, read_len: usize| -> NewLocation {
                //we are already cut to size.
                if location.start + location.len > read_len {
                    NewLocation::Remove
                } else {
                    NewLocation::Keep
                }
            },
            None,
        );
    }

    pub fn filter_tag_locations_all_targets(
        &mut self,
        mut f: impl FnMut(&HitRegion, usize) -> NewLocation,
    ) {
        //possibly we might need this to pass in all 4 reads.
        //but for now, it's only being used by r1/r2 swap.
        for values in self.tags.values_mut() {
            if let TagColumn::Location(values) = values {
                for (ii, tag_val) in values.iter_mut().enumerate() {
                    if let Some(hits) = tag_val {
                        let mut any_none = false;
                        for hit in &mut hits.0 {
                            if let Some(location) = hit.location.as_mut() {
                                match f(location, ii) {
                                    NewLocation::Remove => {
                                        hit.location = None;
                                        any_none = true;
                                        break;
                                    }
                                    NewLocation::Keep => {}
                                    NewLocation::New(new) => *location = new,
                                    // cov:excl-start
                                    NewLocation::NewWithSeq(_new_loc, _new_seq) => {
                                        unreachable!(
                                            "Shouldn't return a new seq when you haven't seen the old.\
                                            you need to change the callback fucntion to also see the seq!"
                                        )
                                        // *location = new_loc;
                                        // hit.sequence = new_seq;
                                    } // cov:excl-stop
                                }
                            }
                        }
                        // if any are no longer present, remove all location spans
                        if any_none {
                            for hit in &mut hits.0 {
                                hit.location = None;
                            }
                        }
                    } else {
                        // no hits, so no location to change
                    }
                }
            }
        }
    }
}

pub struct FastQBlocksCombinedIterator<'a> {
    pos: usize,
    inner: &'a FastQBlocksCombined,
}

pub struct CombinedFastQBlock<'a> {
    pub segments: Vec<WrappedFastQRead<'a>>,
}

impl CombinedFastQBlock<'_> {
    #[must_use]
    pub fn hit_to_qualities(&self, hits: &Hits) -> Option<BString> {
        let mut res = BString::new(Vec::new());
        for hit in &hits.0 {
            let location = hit.location.as_ref()?;
            let seqment_quality = self.segments[location.segment_index.0].qual();
            res.push_str(&seqment_quality[location.start..location.start + location.len]);
        }
        Some(res)
    }
}

impl FastQBlocksCombinedIterator<'_> {
    pub fn pseudo_next(&mut self) -> Option<CombinedFastQBlock<'_>> {
        let len = self.inner.segments[0].entries.len();
        if self.pos >= len || len == 0 {
            return None;
        }
        let segments = self
            .inner
            .segments
            .iter()
            .map(|segment| WrappedFastQRead(&segment.entries[self.pos], &segment.block))
            .collect();

        let e = CombinedFastQBlock { segments };
        self.pos += 1;
        Some(e)
    }
}

pub struct FastQBlocksCombinedIteratorIncludingTag<'a> {
    pos: usize,
    inner: &'a FastQBlocksCombined,
}

impl FastQBlocksCombinedIteratorIncludingTag<'_> {
    pub fn pseudo_next(&mut self) -> Option<(CombinedFastQBlock<'_>, DemultiplexTag)> {
        let len = self.inner.segments[0].entries.len();
        if self.pos >= len || len == 0 {
            return None;
        }
        let segments = self
            .inner
            .segments
            .iter()
            .map(|segment| WrappedFastQRead(&segment.entries[self.pos], &segment.block))
            .collect();

        let tag = match &self.inner.output_tags {
            Some(tags) => tags[self.pos],
            None => 0,
        };
        let e = (CombinedFastQBlock { segments }, tag);
        self.pos += 1;
        Some(e)
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

#[cfg(test)]
mod test {

    #[test]
    fn test_longest_suffix_that_is_a_prefix() {
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTAGCT", b"ACGT", 0, NonZero::new(1).unwrap()),
            None
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACGT", b"ACGT", 0, NonZero::new(1).unwrap()),
            Some(4)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACGC", b"ACGT", 1, NonZero::new(1).unwrap()),
            Some(4)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACGC", b"ACGT", 0, NonZero::new(1).unwrap()),
            None
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACG", b"ACGT", 0, NonZero::new(1).unwrap()),
            Some(3)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTAC", b"ACGT", 0, NonZero::new(1).unwrap()),
            Some(2)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTA", b"ACGT", 0, NonZero::new(1).unwrap()),
            Some(1)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACG", b"ACGT", 0, NonZero::new(1).unwrap()),
            Some(3)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACG", b"ACGT", 0, NonZero::new(3).unwrap()),
            Some(3)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACG", b"ACGT", 0, NonZero::new(4).unwrap()),
            None
        );
    }

    fn get_owned() -> FastQRead {
        FastQRead::new(
            FastQElement::Owned(b"Name".to_vec()),
            FastQElement::Owned(b"ACGTACGTACGT".to_vec()),
            FastQElement::Owned(b"IIIIIIIIIIII".to_vec()),
        )
        .expect("test operation should succeed")
    }

    fn get_local() -> (FastQRead, Vec<u8>) {
        let data = b"@Name\nACGTACGTACGT\n+\nIIIIIIIIIIII\n";
        let res = (
            FastQRead::new(
                FastQElement::Local(Position { start: 1, end: 5 }),
                FastQElement::Local(Position { start: 6, end: 18 }),
                FastQElement::Local(Position { start: 21, end: 33 }),
            )
            .expect("test operation should succeed"),
            data.to_vec(),
        );
        assert_eq!(res.0.seq.get(&res.1), b"ACGTACGTACGT");
        assert_eq!(res.0.qual.get(&res.1), b"IIIIIIIIIIII");
        assert_eq!(res.0.name.get(&res.1), b"Name");
        res
    }

    use super::*;
    #[test]
    fn test_cut_start_owned() {
        let mut input = get_owned();
        input.cut_start(4);
        assert_eq!(input.seq.get(&[]), b"ACGTACGT");
        assert_eq!(input.qual.get(&[]), b"IIIIIIII");
        assert_eq!(input.name.get(&[]), b"Name");
        assert!(!input.qual.is_empty());
        input.cut_start(40);
        assert_eq!(input.seq.get(&[]), b"");
        assert_eq!(input.qual.get(&[]), b"");
        assert_eq!(input.name.get(&[]), b"Name");
        assert!(input.qual.is_empty());
        assert!(!input.name.is_empty());
    }

    #[test]
    fn test_cut_start_local() {
        let (mut input, data) = get_local();
        input.cut_start(2);
        assert_eq!(input.seq.get(&data), b"GTACGTACGT");
        assert_eq!(input.qual.get(&data), b"IIIIIIIIII");
        assert!(!input.qual.is_empty());
        input.cut_start(40);
        assert_eq!(input.seq.get(&data), b"");
        assert_eq!(input.qual.get(&data), b"");
        assert_eq!(input.name.get(&data), b"Name");
        assert!(input.qual.is_empty());
        assert!(!input.name.is_empty());
    }

    #[test]
    fn test_cut_end_owned() {
        let mut input = get_owned();
        input.cut_end(4);
        assert_eq!(input.seq.get(&[]), b"ACGTACGT");
        assert_eq!(input.qual.get(&[]), b"IIIIIIII");
        assert_eq!(input.name.get(&[]), b"Name");
        input.cut_end(40);
        assert_eq!(input.seq.get(&[]), b"");
        assert_eq!(input.qual.get(&[]), b"");
        assert_eq!(input.name.get(&[]), b"Name");
    }

    #[test]
    fn test_cut_end_local() {
        let (mut input, data) = get_local();
        input.cut_end(2);
        assert_eq!(input.seq.get(&data), b"ACGTACGTAC");
        assert_eq!(input.qual.get(&data), b"IIIIIIIIII");
        input.cut_end(40);
        assert_eq!(input.seq.get(&data), b"");
        assert_eq!(input.qual.get(&data), b"");
        assert_eq!(input.name.get(&data), b"Name");
    }

    #[test]
    fn test_maxlen() {
        let (mut input, data) = get_local();
        input.max_len(3);
        assert_eq!(input.seq.get(&data), b"ACG");
        assert_eq!(input.qual.get(&data), b"III");
        input.cut_end(40);
        assert_eq!(input.seq.get(&data), b"");
        assert_eq!(input.qual.get(&data), b"");
        assert_eq!(input.name.get(&data), b"Name");
    }

    #[test]
    fn test_prefix() {
        let (mut input, mut data) = get_local();
        input.seq.prefix(b"TTT", &mut data);
        input.qual.prefix(b"222", &mut data);
        assert_eq!(input.seq.get(&data), b"TTTACGTACGTACGT");
        assert_eq!(input.qual.get(&data), b"222IIIIIIIIIIII");
    }
    #[test]
    fn test_postfix() {
        let (mut input, mut data) = get_local();
        input.seq.postfix(b"TTT", &mut data);
        input.qual.postfix(b"222", &mut data);
        assert_eq!(input.seq.get(&data), b"ACGTACGTACGTTTT");
        assert_eq!(input.qual.get(&data), b"IIIIIIIIIIII222");
    }
    #[test]
    fn test_reverse_owned() {
        let mut input = get_owned();
        let mut data = Vec::new();
        input.seq.prefix(b"T", &mut data);
        input.qual.prefix(b"2", &mut data);
        input.seq.reverse(&mut []);
        input.qual.reverse(&mut []);
        assert_eq!(input.qual.get(&[]), b"IIIIIIIIIIII2");
        assert_eq!(input.seq.get(&[]), b"TGCATGCATGCAT");
    }
    #[test]
    fn test_reverse_local() {
        let (mut input, mut data) = get_local();
        input.seq.prefix(b"T", &mut data);
        input.qual.prefix(b"2", &mut data);
        input.seq.reverse(&mut data);
        input.qual.reverse(&mut data);
        assert_eq!(input.seq.get(&data), b"TGCATGCATGCAT");
        assert_eq!(input.qual.get(&data), b"IIIIIIIIIIII2");
    }

    fn get_owned2(seq: &[u8]) -> FastQRead {
        FastQRead::new(
            FastQElement::Owned(b"Name".to_vec()),
            FastQElement::Owned(seq.to_vec()),
            FastQElement::Owned(vec![b'I'; seq.len()]),
        )
        .expect("test operation should succeed")
    }

    fn get_local2(seq: &[u8]) -> (FastQRead, Vec<u8>) {
        let mut data = b"@Name\n".to_vec();
        data.extend(seq);
        data.extend(b"\n+\n");
        data.extend(vec![b'I'; seq.len()]);
        data.push(b'\n');
        let res = (
            FastQRead {
                name: FastQElement::Local(Position { start: 1, end: 5 }),
                seq: FastQElement::Local(Position {
                    start: 6,
                    end: 6 + seq.len(),
                }),
                qual: FastQElement::Local(Position {
                    start: 6 + seq.len() + 3,
                    end: 6 + seq.len() + 3 + seq.len(),
                }),
            },
            data.clone(),
        );
        assert_eq!(res.0.seq.get(&res.1), seq);
        assert_eq!(res.0.qual.get(&res.1), vec![b'I'; seq.len()]);
        assert_eq!(res.0.name.get(&res.1), b"Name");
        res
    }

    #[test]
    #[expect(clippy::too_many_lines, reason = "needed")]
    fn test_trim_poly_n_local() {
        fn trim(seq: &str, min_length: usize, max_mismatch_fraction: f32, base: u8) -> String {
            let (mut read, mut data) = get_local2(seq.as_bytes());
            let mut read2 = WrappedFastQReadMut(&mut read, &mut data);
            read2.trim_poly_base_suffix(min_length, max_mismatch_fraction, 5, base);
            std::str::from_utf8(read2.seq())
                .expect("test sequence should be valid UTF-8")
                .to_string()
        }
        assert_eq!(&trim("AA", 2, 0.0, b'A'), "");

        assert_eq!(&trim("NNNN", 1, 0.0, b'N'), "");

        assert_eq!(&trim("AGCT", 1, 0.0, b'T'), "AGC");
        assert_eq!(&trim("AGCT", 2, 0.0, b'T'), "AGCT");
        assert_eq!(&trim("AA", 3, 0.0, b'T'), "AA");

        assert_eq!(&trim("AGCT", 1, 0.0, b'G'), "AGCT");
        assert_eq!(&trim("AGCT", 1, 0.0, b'T'), "AGC");
        assert_eq!(&trim("AGCTNNN", 1, 0.0, b'N'), "AGCT");
        assert_eq!(&trim("NGCTNNN", 1, 0.0, b'N'), "NGCT");
        assert_eq!(&trim("NNNN", 1, 0.0, b'.'), "");
        assert_eq!(&trim("AGCTNTN", 1, 1., b'N'), "AGCT");
        assert_eq!(&trim("AGCT", 1, 0.0, b'T'), "AGC");
        assert_eq!(&trim("AGCT", 1, 0.0, b'T'), "AGC");
        assert_eq!(&trim("AGCT", 2, 0.0, b'T'), "AGCT");
        assert_eq!(&trim("ATCT", 2, 1. / 3., b'T'), "A");

        assert_eq!(
            &trim("AAAAAAAAAAAACCCCCCAAAAA", 2, 1. / 3., b'A'),
            "AAAAAAAAAAAACCCCCC"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN",
                24,
                0.0,
                b'N'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN",
                10,
                0.0,
                b'N'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN",
                25,
                0.0,
                b'N'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN",
                24,
                0.0,
                b'.'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT",
                24,
                0.0,
                b'.'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN",
                25,
                0.0,
                b'.'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN"
        );
        //that should both be accepted at 1/24th
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNGNNNNNNNNNNNNNNNNNNNNNN",
                24,
                1. / 24.0,
                b'N'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN",
                24,
                1. / 24.0,
                b'.'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN",
                25,
                1. / 24.0,
                b'.'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN"
        );
    }
    #[test]
    #[expect(clippy::too_many_lines, reason = "long test")]
    fn test_trimm_poly_n() {
        fn trim(seq: &str, min_length: usize, max_mismatch_fraction: f32, base: u8) -> String {
            let mut read = get_owned2(seq.as_bytes());
            let mut data = Vec::new();
            let mut read2 = WrappedFastQReadMut(&mut read, &mut data);
            read2.trim_poly_base_suffix(min_length, max_mismatch_fraction, 5, base);
            std::str::from_utf8(read2.seq())
                .expect("test sequence should be valid UTF-8")
                .to_string()
        }

        assert_eq!(&trim("NNNN", 1, 0.0, b'N'), "");

        assert_eq!(&trim("AGCT", 1, 0.0, b'G'), "AGCT");
        assert_eq!(&trim("AGCT", 1, 0.0, b'T'), "AGC");
        assert_eq!(&trim("AGCTNNN", 1, 0.0, b'N'), "AGCT");
        assert_eq!(&trim("NGCTNNN", 1, 0.0, b'N'), "NGCT");
        assert_eq!(&trim("NNNN", 1, 0.0, b'.'), "");
        assert_eq!(&trim("AGCTNTN", 1, 1., b'N'), "AGCT");
        assert_eq!(&trim("AGCT", 1, 0.0, b'T'), "AGC");
        assert_eq!(&trim("AGCT", 1, 0.0, b'T'), "AGC");
        assert_eq!(&trim("AGCT", 2, 0.0, b'T'), "AGCT");
        assert_eq!(&trim("ATCT", 2, 1. / 3., b'T'), "A");
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN",
                24,
                0.0,
                b'N'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN",
                10,
                0.0,
                b'N'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN",
                25,
                0.0,
                b'N'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN",
                24,
                0.0,
                b'.'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT",
                24,
                0.0,
                b'.'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN",
                25,
                0.0,
                b'.'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN"
        );
        //that should both be accepted at 1/24th
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNGNNNNNNNNNNNNNNNNNNNNNN",
                24,
                1. / 24.0,
                b'N'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN",
                24,
                1. / 24.0,
                b'.'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATG"
        );
        assert_eq!(
            &trim(
                "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN",
                25,
                1. / 24.0,
                b'.'
            ),
            "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN"
        );
        assert_eq!(&trim("ATCCT", 2, 1. / 2., b'.'), "A");
        assert_eq!(&trim("AGCCG", 2, 1. / 2., b'.'), "A");
        assert_eq!(&trim("AACCA", 2, 1. / 2., b'.'), "");
        assert_eq!(&trim("AATTA", 2, 1. / 2., b'.'), "");
    }

    #[test]
    fn test_fastq_block_is_empty() {
        let block = super::FastQBlock {
            block: b"@hello\nagtc\n+\nBBBB".into(),
            entries: vec![],
            first_read_sequential_number: 0,
        };
        assert!(block.is_empty());
        let block = super::FastQBlock {
            block: b"@hello\nagtc\n+\nBBBB".into(),
            entries: vec![super::FastQRead {
                name: super::FastQElement::Owned(b"hello".into()),
                seq: super::FastQElement::Owned(b"agtc".into()),
                qual: super::FastQElement::Owned(b"BBBB".into()),
            }],
            first_read_sequential_number: 0,
        };
        assert!(!block.is_empty());
    }

    #[test]
    fn test_wrapped_fastq_empty() {
        //sinec it's just forwarding to the inner fastq read, on need to test both cases.
        let (read, block) = get_local();
        let wrapped = WrappedFastQRead(&read, &block);
        assert!(!wrapped.is_empty());
        let empty = FastQRead {
            name: FastQElement::Local(Position { start: 0, end: 2 }),
            seq: FastQElement::Local(Position { start: 0, end: 0 }),
            qual: FastQElement::Local(Position { start: 0, end: 0 }),
        };
        let wrapped = WrappedFastQRead(&empty, &block);
        assert!(wrapped.is_empty());
    }

    #[test]
    fn test_replace_qual_local() {
        //longer
        let (mut read, mut block) = get_local();
        let mut wrapped = WrappedFastQReadMut(&mut read, &mut block);
        wrapped.replace_qual(b"IIIIIIIIIIIIIxx"); // longer
        assert!(wrapped.qual().eq(b"IIIIIIIIIIIIIxx"));
        if let FastQElement::Owned(_) = wrapped.0.qual {
            // cov:excl-start
            panic!("Should be local");
            // cov:excl-stop
        }
        //same length
        let (mut read, mut block) = get_local();
        let mut wrapped = WrappedFastQReadMut(&mut read, &mut block);
        let start_len = wrapped.qual().len();
        wrapped.replace_qual(&vec![b'B'; start_len]);
        assert!(wrapped.qual().len() == start_len);
        assert!(wrapped.qual().iter().all(|x| *x == b'B'));
        if let FastQElement::Owned(_) = wrapped.0.qual {
            // cov:excl-start
            panic!("Should not be Owned");
            // cov:excl-stop
        }
        //shorter
        let (mut read, mut block) = get_local();
        let mut wrapped = WrappedFastQReadMut(&mut read, &mut block);
        wrapped.replace_qual(b"xx"); // longer
        assert!(wrapped.qual().eq(b"xx"));
        if let FastQElement::Owned(_) = wrapped.0.qual {
            // cov:excl-start
            panic!("Should not be owned");
            // cov:excl-stop
        }
    }

    // #[test]
    // fn test_trim_adapter_mismatch_tail_early_exit() {
    //     let (mut read, mut block) = get_local();
    //     let (read2, block2) = get_local();
    //     let mut wrapped = WrappedFastQReadMut(&mut read, &mut block);
    //     wrapped.trim_adapter_mismatch_tail(b"AGTCAGTCAGTCA", 12, 1);
    //     assert!(wrapped.seq() == read2.seq.get(&block2));
    // }

    #[test]
    fn test_trim_polybase_min_longer_than_seq() {
        let (mut read, mut block) = get_local();
        let (mut read2, mut block2) = get_local();
        read.seq.replace(b"AAAA", &mut block);
        read2.seq.replace(b"AAAA", &mut block2);
        let mut wrapped = WrappedFastQReadMut(&mut read, &mut block);
        wrapped.trim_poly_base_suffix(25, 0.3, 3, b'A');
        assert!(wrapped.seq() == read2.seq.get(&block2));
    }

    #[test]
    fn test_fastq_block_combined_sanity_check_empty() {
        let empty = FastQBlocksCombined::new(
            vec![FastQBlock::empty()],
            None,
            Default::default(),
            false,
            0,
        );
        empty
            .sanity_check()
            .expect("sanity check should pass in test");
    }
    #[test]
    #[should_panic(expected = "Segment counts differ")]
    fn test_fastq_block_combined_sanity_check_r1_neq_r2() {
        let empty = FastQBlocksCombined::new(
            vec![
                FastQBlock {
                    block: b"hello".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                    first_read_sequential_number: 0,
                },
                FastQBlock::empty(),
                FastQBlock::empty(),
                FastQBlock::empty(),
            ],
            None,
            Default::default(),
            false,
            0,
        );
        empty
            .sanity_check()
            .expect("sanity check should pass in test");
    }

    #[test]
    #[should_panic(expected = "Segment counts differ")]
    fn test_fastq_block_combined_sanity_check_r1_neq_i1() {
        let empty = FastQBlocksCombined::new(
            vec![
                FastQBlock {
                    block: b"hello/1".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                    first_read_sequential_number: 0,
                },
                FastQBlock {
                    block: b"hello/2".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                    first_read_sequential_number: 0,
                },
                FastQBlock::empty(),
                FastQBlock::empty(),
            ],
            None,
            Default::default(),
            false,
            0,
        );
        empty
            .sanity_check()
            .expect("sanity check should pass in test");
    }

    #[test]
    #[should_panic(expected = "Segment counts differ")]
    fn test_fastq_block_combined_sanity_check_r1_neq_i2() {
        let empty = FastQBlocksCombined::new(
            vec![
                FastQBlock {
                    block: b"hello/1".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                    first_read_sequential_number: 0,
                },
                FastQBlock {
                    block: b"hello/2".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                    first_read_sequential_number: 0,
                },
                FastQBlock {
                    block: b"hello/i1".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                    first_read_sequential_number: 0,
                },
                FastQBlock::empty(),
            ],
            None,
            Default::default(),
            false,
            0,
        );
        empty
            .sanity_check()
            .expect("sanity check should pass in test");
    }

    #[test]
    #[should_panic(expected = "Output tag count differs")]
    fn test_fastq_block_combined_sanity_check_r1_eq_output_tags() {
        let empty = FastQBlocksCombined::new(
            vec![
                FastQBlock {
                    block: b"hello/1".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                    first_read_sequential_number: 0,
                },
                FastQBlock {
                    block: b"hello/2".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                    first_read_sequential_number: 0,
                },
                FastQBlock {
                    block: b"hello/i1".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                    first_read_sequential_number: 0,
                },
                FastQBlock {
                    block: b"hello/i2".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                    first_read_sequential_number: 0,
                },
            ],
            Some(vec![]),
            Default::default(),
            false,
            0,
        );
        empty
            .sanity_check()
            .expect("sanity check should pass in test");
    }

    // Tests for FastQElement::swap_with
    #[test]
    fn test_fastq_element_swap_both_local() {
        // Create two local elements with different data
        let mut block1 = b"AAAAAABBBBBB".to_vec();
        let mut block2 = b"CCCCCCDDDDDD".to_vec();

        let mut elem1 = FastQElement::Local(Position { start: 0, end: 6 });
        let mut elem2 = FastQElement::Local(Position { start: 0, end: 6 });

        // Verify initial values
        assert_eq!(elem1.get(&block1), b"AAAAAA");
        assert_eq!(elem2.get(&block2), b"CCCCCC");

        // Swap
        elem1.swap_with(&mut elem2, &mut block1, &mut block2);

        // Verify swapped values
        assert_eq!(elem1.get(&block1), b"CCCCCC");
        assert_eq!(elem2.get(&block2), b"AAAAAA");

        // Verify they're still Local
        assert!(matches!(elem1, FastQElement::Local(_)));
        assert!(matches!(elem2, FastQElement::Local(_)));
    }

    #[test]
    fn test_fastq_element_swap_both_local_self_larger() {
        // self has 8 bytes of slot, other has 4 — both fit in each other's slot
        let mut block1 = b"AAAAAAAA....".to_vec();
        let mut block2 = b"CCCC........".to_vec();

        let mut elem1 = FastQElement::Local(Position { start: 0, end: 8 });
        let mut elem2 = FastQElement::Local(Position { start: 0, end: 4 });

        assert_eq!(elem1.get(&block1), b"AAAAAAAA");
        assert_eq!(elem2.get(&block2), b"CCCC");

        elem1.swap_with(&mut elem2, &mut block1, &mut block2);

        assert_eq!(elem1.get(&block1), b"CCCC");
        assert_eq!(elem2.get(&block2), b"AAAAAAAA");

        // other (4 bytes) fits in self (8) — self stays Local.
        // self (8 bytes) does NOT fit in other (4) — other becomes Owned.
        assert!(matches!(elem1, FastQElement::Local(_)));
        assert!(matches!(elem2, FastQElement::Owned(_)));
    }

    #[test]
    fn test_fastq_element_swap_both_local_other_larger() {
        // Mirror of the previous: self has 4 bytes, other has 8.
        let mut block1 = b"AAAA........".to_vec();
        let mut block2 = b"CCCCCCCC....".to_vec();

        let mut elem1 = FastQElement::Local(Position { start: 0, end: 4 });
        let mut elem2 = FastQElement::Local(Position { start: 0, end: 8 });

        assert_eq!(elem1.get(&block1), b"AAAA");
        assert_eq!(elem2.get(&block2), b"CCCCCCCC");

        elem1.swap_with(&mut elem2, &mut block1, &mut block2);

        assert_eq!(elem1.get(&block1), b"CCCCCCCC");
        assert_eq!(elem2.get(&block2), b"AAAA");

        // other (8 bytes) does NOT fit in self (4) — self becomes Owned.
        // self (4 bytes) fits in other (8) — other stays Local.
        assert!(matches!(elem1, FastQElement::Owned(_)));
        assert!(matches!(elem2, FastQElement::Local(_)));
    }

    #[test]
    fn test_fastq_element_swap_both_owned() {
        let mut elem1 = FastQElement::Owned(b"AAAAAA".to_vec());
        let mut elem2 = FastQElement::Owned(b"CCCCCC".to_vec());
        let mut block1 = Vec::new();
        let mut block2 = Vec::new();

        // Verify initial values
        assert_eq!(elem1.get(&block1), b"AAAAAA");
        assert_eq!(elem2.get(&block2), b"CCCCCC");

        // Swap
        elem1.swap_with(&mut elem2, &mut block1, &mut block2);

        // Verify swapped values
        assert_eq!(elem1.get(&block1), b"CCCCCC");
        assert_eq!(elem2.get(&block2), b"AAAAAA");

        // Verify they're still Owned
        assert!(matches!(elem1, FastQElement::Owned(_)));
        assert!(matches!(elem2, FastQElement::Owned(_)));
    }

    #[test]
    fn test_fastq_element_swap_local_and_owned() {
        let mut block1 = b"AAAAAA".to_vec();
        let mut block2 = Vec::new();

        let mut elem1 = FastQElement::Local(Position { start: 0, end: 6 });
        let mut elem2 = FastQElement::Owned(b"CCCCCC".to_vec());

        // Verify initial values
        assert_eq!(elem1.get(&block1), b"AAAAAA");
        assert_eq!(elem2.get(&block2), b"CCCCCC");

        // Swap
        elem1.swap_with(&mut elem2, &mut block1, &mut block2);

        // Verify swapped values
        assert_eq!(elem1.get(&block1), b"CCCCCC");
        assert_eq!(elem2.get(&block2), b"AAAAAA");

        // After swapping Local with Owned, local fit, so we retain it.
        assert!(matches!(elem1, FastQElement::Local(_)));
        assert!(matches!(elem2, FastQElement::Owned(_)));
    }

    #[test]
    fn test_fastq_element_swap_local_and_owned_larger() {
        let mut block1 = b"BBBBBAAAAAA".to_vec();
        let mut block2 = Vec::new();

        let mut elem1 = FastQElement::Local(Position {
            start: 5,
            end: 6 + 5,
        });
        let mut elem2 = FastQElement::Owned(b"CCCCCCCCCCCCCC".to_vec());

        // Verify initial values
        assert_eq!(elem1.get(&block1), b"AAAAAA");
        assert_eq!(elem2.get(&block2), b"CCCCCCCCCCCCCC");

        // Swap
        elem1.swap_with(&mut elem2, &mut block1, &mut block2);

        // Verify swapped values
        assert_eq!(elem1.get(&block1), b"CCCCCCCCCCCCCC");
        assert_eq!(elem2.get(&block2), b"AAAAAA");

        // After swapping Local with Owned, both should be Owned
        assert!(matches!(elem1, FastQElement::Owned(_)));
        assert!(matches!(elem2, FastQElement::Owned(_)));
    }

    #[test]
    fn test_fastq_element_swap_owned_and_local() {
        let mut block1 = Vec::new();
        let mut block2 = b"CCCCCC".to_vec();

        let mut elem1 = FastQElement::Owned(b"AAAAAA".to_vec());
        let mut elem2 = FastQElement::Local(Position { start: 0, end: 6 });

        // Verify initial values
        assert_eq!(elem1.get(&block1), b"AAAAAA");
        assert_eq!(elem2.get(&block2), b"CCCCCC");

        // Swap
        elem1.swap_with(&mut elem2, &mut block1, &mut block2);

        // Verify swapped values
        assert_eq!(elem1.get(&block1), b"CCCCCC");
        assert_eq!(elem2.get(&block2), b"AAAAAA");

        // After swapping Owned with Local, both should be Owned
        assert!(matches!(elem1, FastQElement::Owned(_)));
        assert!(matches!(elem2, FastQElement::Local(_)));
    }

    #[test]
    fn test_fastq_element_swap_owned_and_local_larger() {
        let mut block1 = Vec::new();
        let mut block2 = b"CCCCCC".to_vec();

        let mut elem1 =
            FastQElement::Owned(b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec());
        let mut elem2 = FastQElement::Local(Position { start: 0, end: 6 });

        // Verify initial values
        assert_eq!(
            elem1.get(&block1),
            b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );
        assert_eq!(elem2.get(&block2), b"CCCCCC");

        // Swap
        elem1.swap_with(&mut elem2, &mut block1, &mut block2);

        // Verify swapped values
        assert_eq!(elem1.get(&block1), b"CCCCCC");
        assert_eq!(
            elem2.get(&block2),
            b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );

        // After swapping Owned with Local, both should be Owned
        assert!(matches!(elem1, FastQElement::Owned(_)));
        assert!(matches!(elem2, FastQElement::Owned(_)));
    }

    #[test]
    fn test_fastq_read_swap_with() {
        // Create two reads with different data
        let mut block1 = b"@read1\nAAAAAAAA\n+\nIIIIIIII\n".to_vec();
        let mut block2 = b"@read2\nCCCCCCCC\n+\nJJJJJJJJ\n".to_vec();

        let mut read1 = FastQRead {
            name: FastQElement::Local(Position { start: 1, end: 6 }),
            seq: FastQElement::Local(Position { start: 7, end: 15 }),
            qual: FastQElement::Local(Position { start: 18, end: 26 }),
        };

        let mut read2 = FastQRead {
            name: FastQElement::Local(Position { start: 1, end: 6 }),
            seq: FastQElement::Local(Position { start: 7, end: 15 }),
            qual: FastQElement::Local(Position { start: 18, end: 26 }),
        };

        // Verify initial values
        assert_eq!(read1.name.get(&block1), b"read1");
        assert_eq!(read1.seq.get(&block1), b"AAAAAAAA");
        assert_eq!(read1.qual.get(&block1), b"IIIIIIII");
        assert_eq!(read2.name.get(&block2), b"read2");
        assert_eq!(read2.seq.get(&block2), b"CCCCCCCC");
        assert_eq!(read2.qual.get(&block2), b"JJJJJJJJ");

        // Swap
        read1.swap_with(&mut read2, &mut block1, &mut block2);

        // Verify swapped values
        assert_eq!(read1.name.get(&block1), b"read2");
        assert_eq!(read1.seq.get(&block1), b"CCCCCCCC");
        assert_eq!(read1.qual.get(&block1), b"JJJJJJJJ");
        assert_eq!(read2.name.get(&block2), b"read1");
        assert_eq!(read2.seq.get(&block2), b"AAAAAAAA");
        assert_eq!(read2.qual.get(&block2), b"IIIIIIII");
    }

    #[test]
    fn test_owned_postfix() {
        let mut seq1 = FastQElement::Owned(b"AAAAAAAA".to_vec());
        seq1.postfix(b"TTT", &mut vec![]);
        assert_eq!(seq1.get(&[]), b"AAAAAAAATTT");
    }

    #[test]
    fn test_cloning() {
        let mixed_block = FastQBlock {
            block: b"@read1\nAAAAAAAA\n+\nIIIIIIII\n".to_vec(),
            entries: vec![
                FastQRead {
                    name: FastQElement::Local(Position { start: 1, end: 6 }),
                    seq: FastQElement::Local(Position { start: 7, end: 15 }),
                    qual: FastQElement::Local(Position { start: 18, end: 26 }),
                },
                FastQRead {
                    name: FastQElement::Owned(b"read2".to_vec()),
                    seq: FastQElement::Owned(b"CCCCCCCC".to_vec()),
                    qual: FastQElement::Owned(b"JJJJJJJJ".to_vec()),
                },
            ],
            first_read_sequential_number: 0,
        };
        let cloned = mixed_block.clone();
        assert_eq!(cloned.block, mixed_block.block);
        assert_eq!(cloned.entries.len(), mixed_block.entries.len());
        assert_eq!(cloned.entries[0].name.get(&cloned.block), b"read1");
        assert_eq!(cloned.entries[0].seq.get(&cloned.block), b"AAAAAAAA");
        assert_eq!(cloned.entries[0].qual.get(&cloned.block), b"IIIIIIII");
        assert_eq!(cloned.entries[1].name.get(&[]), b"read2");
        assert_eq!(cloned.entries[1].seq.get(&[]), b"CCCCCCCC");
        assert_eq!(cloned.entries[1].qual.get(&[]), b"JJJJJJJJ");
    }
}
