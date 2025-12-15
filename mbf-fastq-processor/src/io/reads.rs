use crate::transformations::prelude::DemultiplexTag;
use crate::{
    config::SegmentIndex,
    dna::{Anchor, Hits, TagValue, hamming},
};
use anyhow::{Result, bail};
use std::collections::HashMap;

use super::Range;

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
    #[must_use]
    pub fn to_owned(&self, block: &[u8]) -> Self {
        Self::Owned(self.get(block).to_vec())
    }

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

    pub fn replace<'a>(&'a mut self, new_value: Vec<u8>, block: &'a mut Vec<u8>) {
        match self {
            FastQElement::Owned(_) => {
                *self = FastQElement::Owned(new_value);
            }
            FastQElement::Local(inner) => {
                if inner.end - inner.start >= new_value.len() {
                    inner.end = inner.start + new_value.len();
                    block[inner.start..inner.end].copy_from_slice(&new_value);
                } else {
                    let new_start = block.len();
                    let new_total_len = new_start + new_value.len();
                    // Resize buffer to accommodate old data + new text
                    block.resize(new_total_len, 0);
                    //copy in the new text
                    block[new_start..new_total_len].copy_from_slice(&new_value);

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
        let reversed = crate::dna::reverse_complement_iupac(m);
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
                let self_data = self_block[pos_self.start..pos_self.end].to_vec();
                let other_data = other_block[pos_other.start..pos_other.end].to_vec();

                let self_len = pos_self.end - pos_self.start;
                let other_len = pos_other.end - pos_other.start;

                // Try to reuse self's block space for other's data
                if other_len <= self_len {
                    self_block[pos_self.start..pos_self.start + other_len]
                        .copy_from_slice(&other_data);
                    pos_self.end = pos_self.start + other_len;
                } else {
                    // Doesn't fit, make it owned
                    *self = FastQElement::Owned(other_data);
                }

                // Try to reuse other's block space for self's data
                if self_len <= other_len {
                    other_block[pos_other.start..pos_other.start + self_len]
                        .copy_from_slice(&self_data);
                    pos_other.end = pos_other.start + self_len;
                } else {
                    // Doesn't fit, make it owned
                    *other = FastQElement::Owned(self_data);
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
                let other_len = pos_other.end - pos_other.start;

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

    #[must_use]
    pub fn to_owned(&self, block: &[u8]) -> FastQRead {
        FastQRead {
            name: self.name.to_owned(block),
            seq: self.seq.to_owned(block),
            qual: self.qual.to_owned(block),
        }
    }

    #[track_caller]
    pub fn verify(&self) -> Result<()> {
        if self.seq.len() != self.qual.len() {
            bail!(
                "Sequence and quality must have the same length. Check your input fastq. Wrapped FASTQ is not supported."
            );
        }
        Ok(())
    }

    pub fn cut_start(&mut self, n: usize) {
        self.seq.cut_start(n);
        self.qual.cut_start(n);
        assert_eq!(self.seq.len(), self.qual.len());
    }

    pub fn cut_end(&mut self, n: usize) {
        self.seq.cut_end(n);
        self.qual.cut_end(n);
        assert_eq!(self.seq.len(), self.qual.len());
    }

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

pub struct FastQBlock {
    pub block: Vec<u8>,
    pub entries: Vec<FastQRead>,
}

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
        }
    }
}

impl FastQBlock {
    #[must_use]
    pub fn empty() -> FastQBlock {
        FastQBlock {
            block: Vec::new(),
            entries: Vec::new(),
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
    #[must_use]
    pub fn get_pseudo_iter_including_tag<'a>(
        &'a self,
        output_tags: &'a Option<Vec<crate::demultiplex::Tag>>,
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
        tag: crate::demultiplex::Tag,
        output_tags: &'a Vec<crate::demultiplex::Tag>,
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

    pub fn apply_mut_with_tag(
        &mut self,
        tags: &HashMap<String, Vec<TagValue>>,
        label: &str,
        f: impl Fn(&mut WrappedFastQReadMut, &TagValue),
    ) {
        let tags = tags
            .get(label)
            .expect("Tag not set, should have been caught earlier");
        assert_eq!(
            tags.len(),
            self.entries.len(),
            "Tags and entries must have the same length",
        );
        for (ii, entry) in &mut self.entries.iter_mut().enumerate() {
            let mut wrapped = WrappedFastQReadMut(entry, &mut self.block);
            f(&mut wrapped, &tags[ii]);
        }
    }

    #[must_use]
    pub fn split_at(mut self, target_reads_per_block: usize) -> (FastQBlock, FastQBlock) {
        if self.len() <= target_reads_per_block {
            (self, FastQBlock::empty())
        } else {
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
                    FastQElement::Owned(_) => {
                        unreachable!("Left and write were owned, that shouldn't happen")
                    }
                    FastQElement::Local(position) => position.start,
                },
                FastQElement::Local(position) => position.end,
            };
            for entry in &mut right {
                match &mut entry.name {
                    FastQElement::Owned(_) => {
                        unreachable!()
                    }
                    FastQElement::Local(position) => {
                        position.start -= buffer_split_pos;
                        position.end -= buffer_split_pos;
                    }
                }
                match &mut entry.seq {
                    FastQElement::Owned(_) => {
                        unreachable!()
                    }
                    FastQElement::Local(position) => {
                        position.start -= buffer_split_pos;
                        position.end -= buffer_split_pos;
                    }
                }
                match &mut entry.qual {
                    FastQElement::Owned(_) => {
                        unreachable!()
                    }
                    FastQElement::Local(position) => {
                        position.start -= buffer_split_pos;
                        position.end -= buffer_split_pos;
                    }
                }
            }
            let right_buf = self.block.drain(buffer_split_pos..).collect();
            let left_buf = self.block;
            (
                FastQBlock {
                    block: left_buf,
                    entries: left,
                },
                FastQBlock {
                    block: right_buf,
                    entries: right,
                },
            )
        }
    }

    #[must_use]
    pub fn split_interleaved(self, interleave_count: usize) -> Vec<FastQBlock> {
        assert!(interleave_count > 1);
        let mut outputs = Vec::new();
        for _ in 0..interleave_count {
            outputs.push(FastQBlock {
                block: self.block.clone(),
                entries: Vec::new(),
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
        tag: crate::demultiplex::Tag,
        output_tags: &'a Vec<crate::demultiplex::Tag>,
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
}

pub struct FastQBlockPseudoIterIncludingTag<'a> {
    pos: usize,
    inner: &'a FastQBlock,
    output_tags: &'a Option<Vec<crate::demultiplex::Tag>>,
}

impl<'a> FastQBlockPseudoIterIncludingTag<'a> {
    pub fn pseudo_next(&mut self) -> Option<(WrappedFastQRead<'a>, crate::demultiplex::Tag)> {
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

impl std::fmt::Debug for WrappedFastQRead<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = std::str::from_utf8(self.name()).expect("FASTQ field should be valid UTF-8");
        let seq = std::str::from_utf8(self.seq()).expect("FASTQ field should be valid UTF-8");
        f.write_str(&format!("WrappedFastQRead {{ name: {name}, seq: {seq} }}",))
    }
}

impl std::fmt::Debug for WrappedFastQReadMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = std::str::from_utf8(self.name()).expect("FASTQ field should be valid UTF-8");
        let seq = std::str::from_utf8(self.seq()).expect("FASTQ field should be valid UTF-8");
        f.write_str(&format!(
            "WrappedFastQReadMut {{ name: {name}, seq: {seq} }}",
        ))
    }
}

impl WrappedFastQRead<'_> {
    #[must_use]
    pub fn name(&self) -> &[u8] {
        self.0.name.get(self.1)
    }

    #[must_use]
    pub fn name_without_comment(&self, read_comment_insert_char: u8) -> &[u8] {
        //read comment character to a top level input (i suppose) and have them use this
        let full = self.0.name.get(self.1);
        let pos_of_first_space = full.iter().position(|&x| x == read_comment_insert_char);
        match pos_of_first_space {
            Some(pos) => &full[..pos],
            None => full,
        }
    }

    #[must_use]
    pub fn seq(&self) -> &[u8] {
        self.0.seq.get(self.1)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.seq.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.seq.is_empty()
    }

    #[must_use]
    pub fn qual(&self) -> &[u8] {
        self.0.qual.get(self.1)
    }
    pub fn append_as_fastq(&self, out: &mut Vec<u8>) {
        let name = self.0.name.get(self.1);
        let seq = self.0.seq.get(self.1);
        let qual = self.0.qual.get(self.1);
        out.push(b'@');

        out.extend(name);

        out.push(b'\n');
        out.extend(seq);

        out.extend(b"\n+\n");
        out.extend(qual);
        out.push(b'\n');
    }

    pub fn as_fasta(&self, out: &mut Vec<u8>) {
        let name = self.0.name.get(self.1);
        let seq = self.0.seq.get(self.1);
        out.push(b'>');
        out.extend(name);
        out.push(b'\n');
        out.extend(seq);
        out.push(b'\n');
    }

    #[must_use]
    pub fn find_iupac(
        &self,
        query: &[u8],
        anchor: Anchor,
        max_mismatches: u8,
        target: SegmentIndex,
    ) -> Option<Hits> {
        let seq = self.0.seq.get(self.1);
        crate::dna::find_iupac(seq, query, anchor, max_mismatches, target)
    }

    #[must_use]
    pub fn find_iupac_with_indel(
        &self,
        query: &[u8],
        anchor: Anchor,
        max_mismatches: usize,
        max_indel_bases: usize,
        max_total_edits: Option<usize>,
        target: SegmentIndex,
    ) -> Option<Hits> {
        let seq = self.0.seq.get(self.1);
        crate::dna::find_iupac_with_indel(
            seq,
            query,
            anchor,
            max_mismatches,
            max_indel_bases,
            max_total_edits,
            target,
        )
    }

    fn owned(&self) -> FastQRead {
        FastQRead {
            name: FastQElement::Owned(self.name().to_vec()),
            seq: FastQElement::Owned(self.seq().to_vec()),
            qual: FastQElement::Owned(self.qual().to_vec()),
        }
    }
}

impl WrappedFastQReadMut<'_> {
    #[must_use]
    pub fn name(&self) -> &[u8] {
        self.0.name.get(self.1)
    }
    #[must_use]
    pub fn seq(&self) -> &[u8] {
        self.0.seq.get(self.1)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.seq.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.seq.len() == 0
    }

    #[must_use]
    pub fn seq_mut(&mut self) -> &mut [u8] {
        self.0.seq.get_mut(self.1)
    }

    #[must_use]
    pub fn qual(&self) -> &[u8] {
        self.0.qual.get(self.1)
    }

    /* pub fn name_mut(&mut self) -> &mut [u8] {
        self.0.name.get_mut(self.1)
    }
    pub fn seq_mut(&mut self) -> &mut [u8] {
        self.0.seq.get_mut(self.1)
    }

    pub fn qual_mut(&mut self) -> &mut [u8] {
        self.0.seq.get_mut(self.1)
    } */

    pub fn cut_start(&mut self, n: usize) {
        self.0.cut_start(n);
    }

    pub fn cut_end(&mut self, n: usize) {
        self.0.cut_end(n);
    }

    pub fn max_len(&mut self, n: usize) {
        self.0.max_len(n);
    }

    pub fn prefix(&mut self, seq: &[u8], qual: &[u8]) {
        self.0.seq.prefix(seq, self.1);
        self.0.qual.prefix(qual, self.1);
        assert_eq!(self.0.seq.len(), self.0.qual.len());
    }

    pub fn postfix(&mut self, seq: &[u8], qual: &[u8]) {
        self.0.seq.postfix(seq, self.1);
        self.0.qual.postfix(qual, self.1);
        assert_eq!(self.0.seq.len(), self.0.qual.len());
    }

    pub fn reverse_complement(&mut self) {
        self.0.seq.reverse_complement(self.1);
        self.0.qual.reverse(self.1);
    }

    pub fn replace_seq(&mut self, new_seq: Vec<u8>, new_qual: Vec<u8>) {
        assert!(new_seq.len() == new_qual.len());
        self.0.seq.replace(new_seq, self.1);
        self.0.qual.replace(new_qual, self.1);
    }

    pub fn replace_name(&mut self, new_name: Vec<u8>) {
        self.0.name.replace(new_name, self.1);
    }

    pub fn replace_qual(&mut self, new_qual: Vec<u8>) {
        self.0.qual.replace(new_qual, self.1);
    }

    /// Clear both sequence and quality, leaving them empty
    pub fn clear(&mut self) {
        self.replace_seq(Vec::new(), Vec::new());
    }

    pub fn trim_adapter_mismatch_tail(
        &mut self,
        query: &[u8],
        min_length: usize,
        max_mismatches: usize,
    ) {
        let seq = self.seq();
        if query.len() > seq.len() {
            return;
        }

        if let Some(suffix_len) =
            longest_suffix_that_is_a_prefix(seq, query, max_mismatches, min_length)
        {
            let should = &seq[..seq.len() - suffix_len].to_vec();
            self.0.seq.cut_end(suffix_len);
            assert_eq!(self.seq(), should);
            self.0.qual.cut_end(suffix_len);
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn trim_poly_base_suffix(
        &mut self,
        min_length: usize,
        max_mismatch_fraction: f32,
        max_consecutive_mismatches: usize,
        base: u8,
    ) {
        #[allow(clippy::cast_precision_loss)]
        fn calc_run_length(
            seq: &[u8],
            query: u8,
            min_length: usize,
            max_mismatch_fraction: f32,
            max_consecutive_mismatches: usize,
        ) -> Option<usize> {
            if seq.len() < min_length {
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
                /* dbg!(
                    ii,
                    base,
                    *base == query,
                    matches, mismatches,
                    seq_len,
                    mismatches as f32 / (matches + mismatches) as f32,
                    (mismatches + 1) as f32 / seq_len,
                     consecutive_mismatch_counter,
                     max_consecutive_mismatches,
                );  */

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

    pub fn trim_quality_start(&mut self, min_qual: u8) {
        let mut cut_pos = 0;
        let qual = self.qual();
        for (ii, q) in qual.iter().enumerate() {
            if *q < min_qual {
                cut_pos = ii + 1;
            } else {
                break;
            }
        }
        if cut_pos > 0 {
            self.0.seq.cut_start(cut_pos);
            self.0.qual.cut_start(cut_pos);
        }
    }

    pub fn trim_quality_end(&mut self, min_qual: u8) {
        let qual = self.qual();
        let mut cut_pos = qual.len();
        for q in qual.iter().rev() {
            if *q < min_qual {
                cut_pos -= 1;
            } else {
                break;
            }
        }
        let ql = qual.len();
        if cut_pos < qual.len() {
            self.0.seq.cut_end(ql - cut_pos);
            self.0.qual.cut_end(ql - cut_pos);
        }
    }
}

pub struct SegmentsCombined<T> {
    pub segments: Vec<T>,
}

#[derive(Clone)]
pub struct FastQBlocksCombined {
    pub segments: Vec<FastQBlock>,
    pub output_tags: Option<Vec<crate::demultiplex::Tag>>, // used by Demultiplex
    pub tags: HashMap<String, Vec<TagValue>>,
    pub is_final: bool,
}

impl FastQBlocksCombined {
    /// create an empty one with the same options filled
    #[must_use]
    pub fn empty(&self) -> FastQBlocksCombined {
        FastQBlocksCombined {
            segments: vec![FastQBlock::empty(); self.segments.len()],
            output_tags: if self.output_tags.is_some() {
                Some(Vec::new())
            } else {
                None
            },
            tags: HashMap::default(),
            is_final: self.is_final,
        }
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
        FastQBlocksCombinedIteratorIncludingTag {
            pos: 0,
            inner: self,
        }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.segments[0].entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments[0].entries.is_empty()
    }

    pub fn resize(&mut self, len: usize) {
        for v in &mut self.segments {
            v.entries.resize_with(len, || {
                panic!("Read amplification not expected. Can't resize to larger")
            });
        }
        if let Some(output_tags) = &mut self.output_tags {
            output_tags.resize_with(len, || {
                panic!("Read amplification not expected. Can't resize to larger")
            });
        }
    }

    pub fn drain(&mut self, range: Range<usize>) {
        for v in &mut self.segments {
            v.entries.drain(range.clone());
        }
        if let Some(output_tags) = &mut self.output_tags {
            output_tags.drain(range.clone());
        }
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

    #[allow(clippy::needless_range_loop)] // it's not needless..
    pub fn apply_mut_with_tag<F>(&mut self, label: &str, mut f: F)
    where
        F: for<'a> FnMut(&mut [WrappedFastQReadMut<'a>], &TagValue),
    {
        let tags = self.tags.get(label).expect("Tag must be present, bug");

        for ii in 0..self.segments[0].entries.len() {
            let mut reads: Vec<WrappedFastQReadMut> = Vec::new();
            for v in &mut self.segments {
                reads.push(WrappedFastQReadMut(&mut v.entries[ii], &mut v.block));
            }
            f(&mut reads, &tags[ii]);
            reads.clear();
        }
    }
    #[allow(clippy::needless_range_loop)] // it's not needless..
    pub fn apply_mut_with_tags<F>(&mut self, label: &str, other_label: &str, mut f: F)
    where
        F: for<'a> FnMut(&mut [WrappedFastQReadMut<'a>], &TagValue, &TagValue),
    {
        let tags = self.tags.get(label).expect("Tag must be present, bug");

        let other_tags = self
            .tags
            .get(other_label)
            .expect("Tag must be present, bug");

        for ii in 0..self.segments[0].entries.len() {
            let mut reads: Vec<WrappedFastQReadMut> = Vec::new();
            for v in &mut self.segments {
                reads.push(WrappedFastQReadMut(&mut v.entries[ii], &mut v.block));
            }
            f(&mut reads, &tags[ii], &other_tags[ii]);
            reads.clear();
        }
    }

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
        if let Some(count) = count {
            if let Some(output_tags) = &self.output_tags {
                assert_eq!(
                    count,
                    output_tags.len(),
                    "Output tag count differs, expected {count}, got {}",
                    output_tags.len()
                );
            }
        }
        Ok(())
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
    /// get us a stand alone `FastQRead`
    #[must_use]
    pub fn owned(&self) -> Vec<FastQRead> {
        self.segments.iter().map(WrappedFastQRead::owned).collect()
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
    pub fn pseudo_next(&mut self) -> Option<(CombinedFastQBlock<'_>, crate::demultiplex::Tag)> {
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

#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn longest_suffix_that_is_a_prefix(
    seq: &[u8],
    query: &[u8],
    max_mismatches: usize,
    min_length: usize,
) -> Option<usize> {
    assert!(min_length >= 1);
    let max_len = std::cmp::min(seq.len(), query.len());
    for prefix_len in (min_length..=max_len).rev() {
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
            longest_suffix_that_is_a_prefix(b"ACGTAGCT", b"ACGT", 0, 1),
            None
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACGT", b"ACGT", 0, 1),
            Some(4)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACGC", b"ACGT", 1, 1),
            Some(4)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACGC", b"ACGT", 0, 1),
            None
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACG", b"ACGT", 0, 1),
            Some(3)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTAC", b"ACGT", 0, 1),
            Some(2)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTA", b"ACGT", 0, 1),
            Some(1)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACG", b"ACGT", 0, 1),
            Some(3)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACG", b"ACGT", 0, 3),
            Some(3)
        );
        assert_eq!(
            longest_suffix_that_is_a_prefix(b"ACGTACGTACG", b"ACGT", 0, 4),
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
    fn test_trim_poly_n_local() {
        fn trim(seq: &str, min_length: usize, max_mismatch_fraction: f32, base: u8) -> String {
            let (mut read, mut data) = get_local2(seq.as_bytes());
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
    }
    #[test]
    #[allow(clippy::too_many_lines)]
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
        };
        assert!(block.is_empty());
        let block = super::FastQBlock {
            block: b"@hello\nagtc\n+\nBBBB".into(),
            entries: vec![super::FastQRead {
                name: super::FastQElement::Owned(b"hello".into()),
                seq: super::FastQElement::Owned(b"agtc".into()),
                qual: super::FastQElement::Owned(b"BBBB".into()),
            }],
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
        wrapped.replace_qual(b"IIIIIIIIIIIIIxx".into()); // longer
        assert!(wrapped.qual().eq(b"IIIIIIIIIIIIIxx"));
        if let FastQElement::Owned(_) = wrapped.0.qual {
            panic!("Should be local");
        }
        //same length
        let (mut read, mut block) = get_local();
        let mut wrapped = WrappedFastQReadMut(&mut read, &mut block);
        let start_len = wrapped.qual().len();
        wrapped.replace_qual(vec![b'B'; start_len]);
        assert!(wrapped.qual().len() == start_len);
        assert!(wrapped.qual().iter().all(|x| *x == b'B'));
        if let FastQElement::Owned(_) = wrapped.0.qual {
            panic!("Should not be Owned");
        }
        //shorter
        let (mut read, mut block) = get_local();
        let mut wrapped = WrappedFastQReadMut(&mut read, &mut block);
        wrapped.replace_qual(b"xx".into()); // longer
        assert!(wrapped.qual().eq(b"xx"));
        if let FastQElement::Owned(_) = wrapped.0.qual {
            panic!("Should not be owned");
        }
    }

    #[test]
    fn test_trim_adapter_mismatch_tail_early_exit() {
        let (mut read, mut block) = get_local();
        let (read2, block2) = get_local();
        let mut wrapped = WrappedFastQReadMut(&mut read, &mut block);
        wrapped.trim_adapter_mismatch_tail(b"AGTCAGTCAGTCA", 12, 1);
        assert!(wrapped.seq() == read2.seq.get(&block2));
    }

    #[test]
    fn test_trim_polybase_min_longer_than_seq() {
        let (mut read, mut block) = get_local();
        let (mut read2, mut block2) = get_local();
        read.seq.replace(b"AAAA".to_vec(), &mut block);
        read2.seq.replace(b"AAAA".to_vec(), &mut block2);
        let mut wrapped = WrappedFastQReadMut(&mut read, &mut block);
        wrapped.trim_poly_base_suffix(25, 0.3, 3, b'A');
        assert!(wrapped.seq() == read2.seq.get(&block2));
    }

    #[test]
    fn test_fastq_blocks_combined_empty_is_empty() {
        let blocks = FastQBlocksCombined::empty(&FastQBlocksCombined {
            segments: vec![FastQBlock::empty()],
            output_tags: None,
            tags: Default::default(),
            is_final: false,
        });
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_fastq_block_combined_sanity_check_empty() {
        let empty = FastQBlocksCombined {
            segments: vec![FastQBlock::empty()],
            output_tags: None,
            tags: Default::default(),
            is_final: false,
        };
        empty
            .sanity_check()
            .expect("sanity check should pass in test");
    }
    #[test]
    #[should_panic(expected = "Segment counts differ")]
    fn test_fastq_block_combined_sanity_check_r1_neq_r2() {
        let empty = FastQBlocksCombined {
            segments: vec![
                FastQBlock {
                    block: b"hello".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                },
                FastQBlock::empty(),
                FastQBlock::empty(),
                FastQBlock::empty(),
            ],
            output_tags: None,
            tags: Default::default(),
            is_final: false,
        };
        empty
            .sanity_check()
            .expect("sanity check should pass in test");
    }

    #[test]
    #[should_panic(expected = "Segment counts differ")]
    fn test_fastq_block_combined_sanity_check_r1_neq_i1() {
        let empty = FastQBlocksCombined {
            segments: vec![
                FastQBlock {
                    block: b"hello/1".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                },
                FastQBlock {
                    block: b"hello/2".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                },
                FastQBlock::empty(),
                FastQBlock::empty(),
            ],
            output_tags: None,
            tags: Default::default(),
            is_final: false,
        };
        empty
            .sanity_check()
            .expect("sanity check should pass in test");
    }

    #[test]
    #[should_panic(expected = "Segment counts differ")]
    fn test_fastq_block_combined_sanity_check_r1_neq_i2() {
        let empty = FastQBlocksCombined {
            segments: vec![
                FastQBlock {
                    block: b"hello/1".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                },
                FastQBlock {
                    block: b"hello/2".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                },
                FastQBlock {
                    block: b"hello/i1".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                },
                FastQBlock::empty(),
            ],
            output_tags: None,
            tags: Default::default(),
            is_final: false,
        };
        empty
            .sanity_check()
            .expect("sanity check should pass in test");
    }

    #[test]
    #[should_panic(expected = "Output tag count differs")]
    fn test_fastq_block_combined_sanity_check_r1_eq_output_tags() {
        let empty = FastQBlocksCombined {
            segments: vec![
                FastQBlock {
                    block: b"hello/1".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                },
                FastQBlock {
                    block: b"hello/2".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                },
                FastQBlock {
                    block: b"hello/i1".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                },
                FastQBlock {
                    block: b"hello/i2".to_vec(),
                    entries: vec![FastQRead {
                        name: FastQElement::Owned(b"hello".to_vec()),
                        seq: FastQElement::Owned(b"agtc".to_vec()),
                        qual: FastQElement::Owned(b"ABCD".to_vec()),
                    }],
                },
            ],
            output_tags: Some(vec![]),
            tags: Default::default(),
            is_final: false,
        };
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

        // After swapping Local with Owned, both should be Owned
        assert!(matches!(elem1, FastQElement::Local(_)));
        assert!(matches!(elem2, FastQElement::Owned(_)));
    }

    #[test]
    fn test_fastq_element_swap_local_and_owned_larger() {
        let mut block1 = b"AAAAAA".to_vec();
        let mut block2 = Vec::new();

        let mut elem1 = FastQElement::Local(Position { start: 0, end: 6 });
        let mut elem2 = FastQElement::Owned(b"CCCCCCC".to_vec());

        // Verify initial values
        assert_eq!(elem1.get(&block1), b"AAAAAA");
        assert_eq!(elem2.get(&block2), b"CCCCCCC");

        // Swap
        elem1.swap_with(&mut elem2, &mut block1, &mut block2);

        // Verify swapped values
        assert_eq!(elem1.get(&block1), b"CCCCCCC");
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

        let mut elem1 = FastQElement::Owned(b"AAAAAAA".to_vec());
        let mut elem2 = FastQElement::Local(Position { start: 0, end: 6 });

        // Verify initial values
        assert_eq!(elem1.get(&block1), b"AAAAAAA");
        assert_eq!(elem2.get(&block2), b"CCCCCC");

        // Swap
        elem1.swap_with(&mut elem2, &mut block1, &mut block2);

        // Verify swapped values
        assert_eq!(elem1.get(&block1), b"CCCCCC");
        assert_eq!(elem2.get(&block2), b"AAAAAAA");

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
}
