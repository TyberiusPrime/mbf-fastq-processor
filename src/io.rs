use anyhow::{Context, Result};
use ex::Wrapper;
use std::io::{BufRead, BufReader, ErrorKind, Read, Seek};

use crate::Molecule;

#[derive(Debug)]
pub struct Position {
    start: usize,
    end: usize,
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
    fn is_owned(&self) -> bool {
        match self {
            FastQElement::Owned(_) => true,
            FastQElement::Local(_) => false,
        }
    }

    fn get<'a>(&'a self, block: &'a [u8]) -> &'a [u8] {
        match self {
            FastQElement::Owned(v) => &v[..],
            FastQElement::Local(p) => &block[p.start..p.end],
        }
    }

    fn get_mut<'a>(&'a mut self, block: &'a mut [u8]) -> &'a mut [u8] {
        match self {
            FastQElement::Owned(v) => &mut v[..],
            FastQElement::Local(p) => &mut block[p.start..p.end],
        }
    }

    pub fn len(&self) -> usize {
        match self {
            FastQElement::Owned(v) => v.len(),
            FastQElement::Local(p) => p.end - p.start,
        }
    }

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

    fn prefix(&mut self, text: &[u8], local_buffer: &[u8]) {
        let mut new = Vec::new();
        new.extend(text);
        new.extend(self.get(local_buffer));
        *self = FastQElement::Owned(new);
    }

    fn postfix(&mut self, text: &[u8], local_buffer: &[u8]) {
        match self {
            FastQElement::Owned(inner) => inner.extend(text),
            FastQElement::Local(_) => {
                let mut new = Vec::new();
                new.extend(self.get(local_buffer));
                new.extend(text);
                *self = FastQElement::Owned(new);
            }
        }
    }

    fn reverse(&mut self, local_buffer: &mut [u8]) {
        self.get_mut(local_buffer).reverse()
    }
}

pub struct FastQRead {
    pub name: FastQElement,
    pub seq: FastQElement,
    pub qual: FastQElement,
}

impl FastQRead {
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
}

pub struct FastQBlock {
    pub block: Vec<u8>,
    pub entries: Vec<FastQRead>,
}

impl FastQBlock {
    fn empty() -> FastQBlock {
        FastQBlock {
            block: Vec::new(),
            entries: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get_pseudo_iter<'a>(&'a self) -> FastQBlockPseudoIter<'a> {
        FastQBlockPseudoIter {
            pos: 0,
            inner: self,
        }
    }

    pub fn apply<T>(&self, f: impl Fn(&mut WrappedFastQRead) -> T) -> Vec<T> {
        let mut res = Vec::new();
        for entry in self.entries.iter() {
            let mut wrapped = WrappedFastQRead(entry, &self.block);
            res.push(f(&mut wrapped));
        }
        res
    }

    pub fn apply_mut(&mut self, f: impl Fn(&mut WrappedFastQReadMut)) {
        for entry in self.entries.iter_mut() {
            let mut wrapped = WrappedFastQReadMut(entry, &mut self.block);
            f(&mut wrapped);
        }
    }

    fn split_at(mut self, target_reads_per_block: usize) -> (FastQBlock, FastQBlock) {
        if self.len() <= target_reads_per_block {
            return (self, FastQBlock::empty());
        } else {
            let mut right: Vec<FastQRead> = self.entries.drain(target_reads_per_block..).collect();
            let left = self.entries;
            //let (left, right) = self.entries.split_at(target_reads_per_block);
            let buffer_split_pos = match &left.iter().last().unwrap().qual {
                FastQElement::Owned(_) => match &right.iter().next().unwrap().name {
                    FastQElement::Owned(_) => {
                        panic!("Left and write were owned, that shouldn't happen")
                    }
                    FastQElement::Local(position) => position.start,
                },
                FastQElement::Local(position) => position.end,
            };
            for entry in right.iter_mut() {
                match &mut entry.name {
                    FastQElement::Owned(_) => {}
                    FastQElement::Local(position) => {
                        position.start -= buffer_split_pos;
                        position.end -= buffer_split_pos;
                    }
                }
                match &mut entry.seq {
                    FastQElement::Owned(_) => {}
                    FastQElement::Local(position) => {
                        position.start -= buffer_split_pos;
                        position.end -= buffer_split_pos;
                    }
                }
                match &mut entry.qual {
                    FastQElement::Owned(_) => {}
                    FastQElement::Local(position) => {
                        position.start -= buffer_split_pos;
                        position.end -= buffer_split_pos;
                    }
                }
            }
            let right_buf = self.block.drain(buffer_split_pos..).collect();
            let left_buf = self.block;
            return (
                FastQBlock {
                    block: left_buf,
                    entries: left,
                },
                FastQBlock {
                    block: right_buf,
                    entries: right,
                },
            );
        }
    }
}

pub struct FastQBlockPseudoIter<'a> {
    pos: usize,
    inner: &'a FastQBlock,
}

impl<'a> FastQBlockPseudoIter<'a> {
    pub fn next(&mut self) -> Option<WrappedFastQRead<'a>> {
        let len = self.inner.entries.len();
        if self.pos >= len || len == 0 {
            return None;
        };
        let e = WrappedFastQRead(&self.inner.entries[self.pos], &self.inner.block);
        self.pos += 1;
        return Some(e);
    }
}

pub struct WrappedFastQReadMut<'a>(&'a mut FastQRead, &'a mut Vec<u8>);
pub struct WrappedFastQRead<'a>(&'a FastQRead, &'a Vec<u8>);

impl std::fmt::Debug for WrappedFastQReadMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = std::str::from_utf8(self.name()).unwrap();
        let seq = std::str::from_utf8(self.seq()).unwrap();
        //let qual = std::str::from_utf8(self.qual()).unwrap();
        f.write_str(&format!(
            "WrappedFastQReadMut {{ name: {}, seq: {} }}",
            name, seq
        ))
    }
}

impl<'a> WrappedFastQRead<'a> {
    pub fn name(&self) -> &[u8] {
        self.0.name.get(&self.1)
    }
    pub fn seq(&self) -> &[u8] {
        self.0.seq.get(&self.1)
    }
    pub fn qual(&self) -> &[u8] {
        self.0.qual.get(&self.1)
    }
    pub fn append_as_fastq(&self, out: &mut Vec<u8>) {
        let name = self.0.name.get(&self.1);
        let seq = self.0.seq.get(&self.1);
        let qual = self.0.qual.get(&self.1);
        out.push(b'@');
        out.extend(name);
        out.push(b'\n');
        out.extend(seq);
        out.extend(b"\n+\n");
        out.extend(qual);
        out.push(b'\n');
    }
}

impl<'a> WrappedFastQReadMut<'a> {
    pub fn name(&self) -> &[u8] {
        self.0.name.get(&self.1)
    }
    pub fn seq(&self) -> &[u8] {
        self.0.seq.get(&self.1)
    }
    pub fn qual(&self) -> &[u8] {
        self.0.qual.get(&self.1)
    }

    pub fn name_mut(&mut self) -> &mut [u8] {
        self.0.name.get_mut(&mut self.1)
    }
    pub fn seq_mut(&mut self) -> &mut [u8] {
        self.0.seq.get_mut(&mut self.1)
    }

    pub fn qual_mut(&mut self) -> &mut [u8] {
        self.0.seq.get_mut(&mut self.1)
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

    pub fn reverse(&mut self) {
        self.0.seq.reverse(self.1);
        self.0.qual.reverse(self.1);
    }

    pub fn replace_name(&mut self, new_name: Vec<u8>) {
        self.0.name = FastQElement::Owned(new_name);
    }

    pub fn replace_qual(&mut self, new_qual: Vec<u8>) {
        match &self.0.qual {
            FastQElement::Owned(_) => {
                self.0.qual = FastQElement::Owned(new_qual);
            }
            FastQElement::Local(old) => {
                if old.end - old.start == new_qual.len() {
                    let buf = &mut self.1;
                    buf[old.start..old.end].copy_from_slice(&new_qual);
                } else {
                    self.0.qual = FastQElement::Owned(new_qual);
                }
            }
        }
    }

    pub fn trim_poly_base(
        &mut self,
        min_length: usize,
        max_mismatch_fraction: f32,
        max_mismatches: usize,
        base: u8,
    ) {
        fn calc_run_length(
            seq: &[u8],
            query: u8,
            min_length: usize,
            max_mismatch_fraction: f32,
            max_mismatches: usize,
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
            // if no position fulfills the above, return None
            let mut matches = 0;
            let mut mismatches = 0;
            let mut last_base_pos = None;
            let seq_len = seq.len() as f32;
            for (ii, base) in seq.iter().enumerate().rev() {
                if *base == query {
                    matches += 1;
                    if seq.len() - ii >= min_length
                        && mismatches as f32 / (matches + mismatches) as f32
                            <= max_mismatch_fraction
                    {
                        last_base_pos = Some(ii);
                    }
                } else {
                    mismatches += 1;
                    if mismatches as f32 / seq_len > max_mismatch_fraction {
                        break;
                    }
                }
            }
            last_base_pos
            //
        }
        let seq = self.seq();

        let last_pos = if base == b'.' {
            let lp_a = calc_run_length(
                &seq,
                b'A',
                min_length,
                max_mismatch_fraction,
                max_mismatches,
            );
            let lp_g = calc_run_length(
                &seq,
                b'G',
                min_length,
                max_mismatch_fraction,
                max_mismatches,
            );
            let lp_c = calc_run_length(
                &seq,
                b'C',
                min_length,
                max_mismatch_fraction,
                max_mismatches,
            );
            let lp_t = calc_run_length(
                &seq,
                b'T',
                min_length,
                max_mismatch_fraction,
                max_mismatches,
            );
            let lp_n = calc_run_length(
                &seq,
                b'N',
                min_length,
                max_mismatch_fraction,
                max_mismatches,
            );
            //now I need to find the right most one that is not None
            let mut lp = lp_a;
            if lp_g.is_some() && (lp.is_none() || lp_g.unwrap() > lp.unwrap()) {
                lp = lp_g;
            }
            if lp_c.is_some() && (lp_c.is_none() || lp_c.unwrap() > lp.unwrap()) {
                lp = lp_c;
            }
            if lp_t.is_some() && (lp.is_none() || lp_t.unwrap() > lp.unwrap()) {
                lp = lp_t;
            }
            if lp_n.is_some() && (lp.is_none() || lp_n.unwrap() > lp.unwrap()) {
                lp = lp_n;
            }
            lp
        } else {
            calc_run_length(
                &seq,
                base,
                min_length,
                max_mismatch_fraction,
                max_mismatches,
            )
        };
        if let Some(last_pos) = last_pos {
            let from_end = seq.len() - last_pos;
            self.0.seq.cut_end(from_end);
            self.0.qual.cut_end(from_end);
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
        for (ii, q) in qual.iter().rev().enumerate() {
            dbg!((ii, *q, *q < min_qual));
            if *q < min_qual {
                cut_pos -= 1;
            } else {
                break;
            }
        }
        dbg!(cut_pos);
        let ql = qual.len();
        if cut_pos < qual.len() {
            self.0.seq.cut_end(ql - cut_pos);
            self.0.qual.cut_end(ql - cut_pos);
        }
    }
}

pub struct FastQBlocksCombined {
    pub block_read1: FastQBlock,
    pub block_read2: Option<FastQBlock>,
    pub block_index1: Option<FastQBlock>,
    pub block_index2: Option<FastQBlock>,
}

impl FastQBlocksCombined {
    /// create an empty one with the same options filled
    pub fn empty(&self) -> FastQBlocksCombined {
        FastQBlocksCombined {
            block_read1: FastQBlock::empty(),
            block_read2: self.block_read2.as_ref().map(|_| FastQBlock::empty()),
            block_index1: self.block_index1.as_ref().map(|_| FastQBlock::empty()),
            block_index2: self.block_index2.as_ref().map(|_| FastQBlock::empty()),
        }
    }
    pub fn get_pseudo_iter<'a>(&'a self) -> FastQBlocksCombinedIterator<'a> {
        FastQBlocksCombinedIterator {
            pos: 0,
            inner: self,
        }
    }

    pub fn len(&self) -> usize {
        return self.block_read1.entries.len();
    }

    pub fn resize(&mut self, len: usize) {
        self.block_read1.entries.resize_with(len, || {
            panic!("Read amplification not expected. Can't resize to larger")
        });
        if let Some(block) = &mut self.block_read2 {
            block.entries.resize_with(len, || {
                panic!("Read amplification not expected. Can't resize to larger")
            });
        }
        if let Some(block) = &mut self.block_index1 {
            block.entries.resize_with(len, || {
                panic!("Read amplification not expected. Can't resize to larger")
            });
        }
        if let Some(block) = &mut self.block_index2 {
            block.entries.resize_with(len, || {
                panic!("Read amplification not expected. Can't resize to larger")
            });
        }
    }

    pub fn apply_mut<F>(&mut self, f: F)
    where
        F: for<'a> Fn(
            &mut WrappedFastQReadMut<'a>,
            &mut Option<&mut WrappedFastQReadMut<'a>>,
            &mut Option<&mut WrappedFastQReadMut<'a>>,
            &mut Option<&mut WrappedFastQReadMut<'a>>,
        ),
    {
        for ii in 0..self.block_read1.entries.len() {
            let mut read1 = WrappedFastQReadMut(
                &mut self.block_read1.entries[ii],
                &mut self.block_read1.block,
            );
            let mut read2 = self
                .block_read2
                .as_mut()
                .map(|x| WrappedFastQReadMut(&mut x.entries[ii], &mut x.block));
            let mut index1 = self
                .block_index1
                .as_mut()
                .map(|x| WrappedFastQReadMut(&mut x.entries[ii], &mut x.block));
            let mut index2 = self
                .block_index2
                .as_mut()
                .map(|x| WrappedFastQReadMut(&mut x.entries[ii], &mut x.block));
            f(
                &mut read1,
                &mut read2.as_mut(),
                &mut index1.as_mut(),
                &mut index2.as_mut(),
            );
        }
    }
}

pub struct FastQBlocksCombinedIterator<'a> {
    pos: usize,
    inner: &'a FastQBlocksCombined,
}

impl<'a> FastQBlocksCombinedIterator<'a> {
    pub fn next(
        &mut self,
    ) -> Option<(
        WrappedFastQRead<'a>,
        Option<WrappedFastQRead<'a>>,
        Option<WrappedFastQRead<'a>>,
        Option<WrappedFastQRead<'a>>,
    )> {
        let len = self.inner.block_read1.entries.len();
        if self.pos >= len || len == 0 {
            return None;
        }

        let e = (
            WrappedFastQRead(
                &self.inner.block_read1.entries[self.pos],
                &self.inner.block_read1.block,
            ),
            self.inner
                .block_read2
                .as_ref()
                .map(|x| WrappedFastQRead(&x.entries[self.pos], &x.block)),
            self.inner
                .block_index1
                .as_ref()
                .map(|x| WrappedFastQRead(&x.entries[self.pos], &x.block)),
            self.inner
                .block_index2
                .as_ref()
                .map(|x| WrappedFastQRead(&x.entries[self.pos], &x.block)),
        );
        self.pos += 1;
        return Some(e);
    }
}

pub struct FastQBlocksCombinedIteratorMut<'a> {
    pos: usize,
    inner: &'a mut FastQBlocksCombined,
}

impl<'a> FastQBlocksCombinedIteratorMut<'a> {
    pub fn next(
        &'a mut self,
    ) -> Option<(
        WrappedFastQReadMut<'a>,
        Option<WrappedFastQReadMut<'a>>,
        Option<WrappedFastQReadMut<'a>>,
        Option<WrappedFastQReadMut<'a>>,
    )> {
        if self.pos >= self.inner.block_read1.entries.len() {
            return None;
        }

        let e = (
            WrappedFastQReadMut(
                &mut self.inner.block_read1.entries[self.pos],
                &mut self.inner.block_read1.block,
            ),
            self.inner
                .block_read2
                .as_mut()
                .map(|x| WrappedFastQReadMut(&mut x.entries[self.pos], &mut x.block)),
            self.inner
                .block_index1
                .as_mut()
                .map(|x| WrappedFastQReadMut(&mut x.entries[self.pos], &mut x.block)),
            self.inner
                .block_index2
                .as_mut()
                .map(|x| WrappedFastQReadMut(&mut x.entries[self.pos], &mut x.block)),
        );
        self.pos += 1;
        return Some(e);
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PartialStatus {
    NoPartial,
    InName,
    InSeq,
    InSpacer,
    InQual,
}

pub struct FastQBlockParseResult {
    //pub block: FastQBlock,
    pub status: PartialStatus,
    pub partial_read: Option<FastQRead>,
}

pub fn parse_to_fastq_block(
    target_block: &mut FastQBlock,
    start_offset: usize,
    last_status: PartialStatus,
    last_read: Option<FastQRead>,
) -> Result<FastQBlockParseResult> {
    let input = &mut target_block.block;
    let entries = &mut target_block.entries;
    let mut pos = start_offset;
    //println!("start offset is {pos}");
    let mut last_status = last_status;
    let mut last_read = last_read;
    //continue where we left off
    if last_status == PartialStatus::InName {
        let last_read = last_read.as_mut().unwrap();
        let next_newline = memchr::memchr(b'\n', &input[pos..]).expect("Truncated fastq?");
        // println!( "Continue reading name: {next_newline} {} {}", input.len(), std::str::from_utf8(&input[..next_newline]).unwrap());
        match &mut last_read.name {
            FastQElement::Owned(name) => {
                name.extend_from_slice(&input[pos..pos + next_newline]);
            }
            FastQElement::Local(_) => panic!("Should not happen"),
        }
        pos = pos + next_newline + 1;
        last_status = PartialStatus::InSeq;
    }
    if PartialStatus::InSeq == last_status {
        let last_read = last_read.as_mut().unwrap();
        let next_newline = memchr::memchr(b'\n', &input[pos..]).expect("Truncated fastq?");
        // println!( "Continue reading seq: {next_newline} {} {}", input.len(), std::str::from_utf8(&input[pos..pos + next_newline]).unwrap());
        match &mut last_read.seq {
            FastQElement::Owned(seq) => {
                seq.extend_from_slice(&input[pos..pos + next_newline]);
            }
            FastQElement::Local(_) => panic!("Should not happen"),
        }
        pos = pos + next_newline + 1;
        last_status = PartialStatus::InSpacer;
    }
    if PartialStatus::InSpacer == last_status {
        let next_newline = memchr::memchr(b'\n', &input[pos..]).expect("Truncated fastq?");
        // println!( "Continue reading spacer: {next_newline} {} {}", input.len(), std::str::from_utf8(&input[pos..pos + next_newline]).unwrap());
        pos = pos + next_newline + 1;
        last_status = PartialStatus::InQual;
    }
    if PartialStatus::InQual == last_status {
        let last_read = last_read.as_mut().unwrap();
        let next_newline = memchr::memchr(b'\n', &input[pos..]).expect("Truncated fastq?");
        // println!( "Continue reading qual: {next_newline} {} {}", input.len(), std::str::from_utf8(&input[pos..pos + next_newline]).unwrap());
        match &mut last_read.qual {
            FastQElement::Owned(qual) => {
                qual.extend_from_slice(&input[pos..pos + next_newline]);
            }
            FastQElement::Local(_) => panic!("Should not happen"),
        }
        pos = pos + next_newline + 1;
    }
    if let Some(last_read) = last_read {
        entries.push(last_read);
    }

    //read full reads until last (possibly partial red)

    let mut status = PartialStatus::NoPartial;
    let mut partial_read = None;

    loop {
        if pos >= input.len() {
            break;
        }
        let end_of_name = memchr::memchr(b'\n', &input[pos..]);
        let (name_start, name_end) = match end_of_name {
            Some(end_of_name) => {
                let r = (pos + 1, end_of_name + pos);
                pos = pos + end_of_name + 1;
                r
            }
            None => {
                status = PartialStatus::InName;
                partial_read = Some(FastQRead {
                    name: FastQElement::Owned(input[pos + 1..].to_vec()),
                    seq: FastQElement::Owned(Vec::new()),
                    qual: FastQElement::Owned(Vec::new()),
                });
                break;
            }
        };
        let end_of_seq = memchr::memchr(b'\n', &input[pos..]);
        let (seq_start, seq_end) = match end_of_seq {
            Some(end_of_seq) => {
                let r = (pos, end_of_seq + pos);
                pos = pos + end_of_seq + 1;
                r
            }
            None => {
                status = PartialStatus::InSeq;
                partial_read = Some(FastQRead {
                    name: FastQElement::Owned(input[name_start..name_end].to_vec()),
                    seq: FastQElement::Owned(input[pos..].to_vec()),
                    qual: FastQElement::Owned(Vec::new()),
                });
                break;
            }
        };
        let end_of_spacer = memchr::memchr(b'\n', &input[pos..]);
        match end_of_spacer {
            Some(end_of_spacer) => {
                pos = pos + end_of_spacer + 1;
            }
            None => {
                status = PartialStatus::InSpacer;
                partial_read = Some(FastQRead {
                    name: FastQElement::Owned(input[name_start..name_end].to_vec()),
                    seq: FastQElement::Owned(input[seq_start..seq_end].to_vec()),
                    qual: FastQElement::Owned(Vec::new()),
                });
                break;
            }
        };
        let end_of_qual = memchr::memchr(b'\n', &input[pos..]);
        let (qual_start, qual_end) = match end_of_qual {
            Some(end_of_qual) => {
                let r = (pos, end_of_qual + pos);
                pos = pos + end_of_qual + 1;
                r
            }
            None => {
                status = PartialStatus::InQual;
                partial_read = Some(FastQRead {
                    name: FastQElement::Owned(input[name_start..name_end].to_vec()),
                    seq: FastQElement::Owned(input[seq_start..seq_end].to_vec()),
                    qual: FastQElement::Owned(input[pos..].to_vec()),
                });
                break;
            }
        };
        entries.push(FastQRead {
            name: FastQElement::Local(Position {
                start: name_start,
                end: name_end,
            }),
            seq: FastQElement::Local(Position {
                start: seq_start,
                end: seq_end,
            }),
            qual: FastQElement::Local(Position {
                start: qual_start,
                end: qual_end,
            }),
        });
    }
    /* let mut owned_count = 0;
    for e in entries.iter() {
        if e.name.is_owned() || e.seq.is_owned() || e.qual.is_owned() {
            owned_count += 1;
        }
    }
    dbg!(owned_count); */

    Ok(FastQBlockParseResult {
        status,
        partial_read,
    })
}

pub struct FastQParser<'a> {
    readers: Vec<NifflerReader<'a>>,
    current_reader: usize,
    current_block: Option<FastQBlock>,
    buf_size: usize,
    target_reads_per_block: usize,
    last_partial: Option<FastQRead>,
    last_status: PartialStatus,
}

impl<'a> FastQParser<'a> {
    pub fn new(
        readers: Vec<NifflerReader<'a>>,
        target_reads_per_block: usize,
        buf_size: usize,
    ) -> FastQParser<'a> {
        FastQParser {
            readers,
            current_reader: 0,
            current_block: Some(FastQBlock {
                block: Vec::new(),
                entries: Vec::new(),
            }),
            buf_size, // for starters.
            target_reads_per_block,
            last_partial: None,
            last_status: PartialStatus::NoPartial,
        }
    }

    pub fn parse(&mut self) -> Result<(FastQBlock, bool)> {
        let mut was_final = false;
        while self.current_block.as_ref().unwrap().entries.len() < self.target_reads_per_block {
            //extend the buf.
            let start = self.current_block.as_ref().unwrap().block.len();
            self.current_block
                .as_mut()
                .unwrap()
                .block
                .extend(vec![0; self.buf_size]);
            // parse the data.
            let read = self.readers[self.current_reader]
                .read(&mut self.current_block.as_mut().unwrap().block[start..])?;
            if read == 0 {
                //println!("advancing file");
                self.current_reader += 1;
                if self.current_reader >= self.readers.len() {
                    //println!("beyond final file");
                    was_final = true;
                    break;
                }
            }
            //println!("read {} bytes", read);
            self.current_block
                .as_mut()
                .unwrap()
                .block
                .resize(start + read, 0);
            // read more data
            let parse_result = parse_to_fastq_block(
                &mut self.current_block.as_mut().unwrap(),
                start,
                self.last_status,
                self.last_partial.take(),
            )?;
            /* if self.current_block.as_ref().unwrap().entries.len() < self.target_reads_per_block {
                self.buf_size = (self.buf_size as f32 * 1.1) as usize;
                println!(
                    "Only read {} entries.read was {read}",
                    self.current_block.as_ref().unwrap().entries.len()
                );
                println!("Increasing buf size to {}", self.buf_size);
            } */
            /* println!(
                "Extended parsed reads to {:?}",
                self.current_block.as_ref().unwrap().entries.len()
            ); */
            self.last_status = parse_result.status;
            self.last_partial = parse_result.partial_read;
            /* println!(
                "last status: {:?}. Read {:?}",
                self.last_status,
                match self.last_partial.as_ref() {
                    Some(inner) => match &inner.name {
                        FastQElement::Owned(x) => std::str::from_utf8(x).unwrap(),
                        FastQElement::Local(_) => panic!("Should not happen"),
                    },
                    None => {
                        "no partial"
                    }
                }
            ); */
        }
        //now we need to cut it *down* to  target_reads_per_block
        let (out_block, new_block) = self
            .current_block
            .take()
            .unwrap()
            .split_at(self.target_reads_per_block);
        /* println!(
            "split into reads {} {} {} {}",
            out_block.len(),
            out_block.entries.len(),
            new_block.len(),
            new_block.entries.len()
        );
        for read in out_block.entries.iter() {
            println!(
                "Left  : {}",
                std::str::from_utf8(read.name.get(&out_block.block)).unwrap(),
            );
        }
        for read in new_block.entries.iter() {
            println!(
                "Right : {}",
                std::str::from_utf8(read.name.get(&new_block.block)).unwrap(),
            );
        } */

        self.current_block = Some(new_block);
        return Ok((out_block, was_final));
    }
}

pub type NifflerReader<'a> = Box<dyn Read + 'a + Send>;

pub struct InputSet<'a> {
    read1: NifflerReader<'a>,
    read2: Option<NifflerReader<'a>>,
    index1: Option<NifflerReader<'a>>,
    index2: Option<NifflerReader<'a>>,
}

pub struct InputFiles<'a> {
    sets: Vec<InputSet<'a>>,
}

impl<'a> InputFiles<'a> {
    pub fn transpose(
        self,
    ) -> (
        Vec<NifflerReader<'a>>,
        Option<Vec<NifflerReader<'a>>>,
        Option<Vec<NifflerReader<'a>>>,
        Option<Vec<NifflerReader<'a>>>,
    ) {
        let mut read1 = Vec::new();
        let mut read2 = Vec::new();
        let mut index1 = Vec::new();
        let mut index2 = Vec::new();
        for set in self.sets {
            read1.push(set.read1);
            if let Some(set_read2) = set.read2 {
                read2.push(set_read2);
            }
            if let Some(set_index1) = set.index1 {
                index1.push(set_index1);
            }
            if let Some(set_index2) = set.index2 {
                index2.push(set_index2);
            }
        }
        (
            read1,
            if read2.is_empty() { None } else { Some(read2) },
            if index1.is_empty() {
                None
            } else {
                Some(index1)
            },
            if index2.is_empty() {
                None
            } else {
                Some(index2)
            },
        )
    }
}

pub fn open_file(filename: &str) -> Result<Box<dyn Read + Send>> {
    let fh = std::fs::File::open(filename).context(format!("Could not open file {}", filename))?;
    let wrapped = niffler::send::get_reader(Box::new(fh))?;
    Ok(wrapped.0)
}

pub fn open_input_files<'a>(input_config: crate::config::ConfigInput) -> Result<InputFiles<'a>> {
    let mut sets = Vec::new();
    for (ii, read1_filename) in (&input_config.read1).into_iter().enumerate() {
        // we can assume all the others are either of the same length, or None
        let read1 = open_file(read1_filename)?;
        let read2 = input_config.read2.as_ref().map(|x| open_file(&x[ii]));
        //bail if it's an Error
        let read2 = match read2 {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => Err(e)?,
            None => None,
        };
        let index1 = input_config.index1.as_ref().map(|x| open_file(&x[ii]));
        let index1 = match index1 {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => Err(e)?,
            None => None,
        };
        let index2 = input_config.index2.as_ref().map(|x| open_file(&x[ii]));
        let index2 = match index2 {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => Err(e)?,
            None => None,
        };
        sets.push(InputSet {
            read1,
            read2,
            index1,
            index2,
        });
    }

    Ok(InputFiles { sets })
}

#[cfg(test)]
mod test {

    fn get_owned() -> FastQRead {
        FastQRead {
            name: FastQElement::Owned(b"Name".to_vec()),
            seq: FastQElement::Owned(b"ACGTACGTACGT".to_vec()),
            qual: FastQElement::Owned(b"IIIIIIIIIIII".to_vec()),
        }
    }

    fn get_local() -> (FastQRead, Vec<u8>) {
        let data = b"@Name\nACGTACGTACGT\n+\nIIIIIIIIIIII\n";
        let res = (
            FastQRead {
                name: FastQElement::Local(Position { start: 1, end: 5 }),
                seq: FastQElement::Local(Position { start: 6, end: 18 }),
                qual: FastQElement::Local(Position { start: 21, end: 33 }),
            },
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
        input.cut_start(40);
        assert_eq!(input.seq.get(&[]), b"");
        assert_eq!(input.qual.get(&[]), b"");
        assert_eq!(input.name.get(&[]), b"Name");
    }

    #[test]
    fn test_cut_start_local() {
        let (mut input, data) = get_local();
        input.cut_start(2);
        assert_eq!(input.seq.get(&data), b"GTACGTACGT");
        assert_eq!(input.qual.get(&data), b"IIIIIIIIII");
        input.cut_start(40);
        assert_eq!(input.seq.get(&data), b"");
        assert_eq!(input.qual.get(&data), b"");
        assert_eq!(input.name.get(&data), b"Name");
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
        let (mut input, data) = get_local();
        input.seq.prefix(b"TTT", &data);
        input.qual.prefix(b"222", &data);
        assert_eq!(input.seq.get(&data), b"TTTACGTACGTACGT");
        assert_eq!(input.qual.get(&data), b"222IIIIIIIIIIII");
    }
    #[test]
    fn test_postfix() {
        let (mut input, data) = get_local();
        input.seq.postfix(b"TTT", &data);
        input.qual.postfix(b"222", &data);
        assert_eq!(input.seq.get(&data), b"ACGTACGTACGTTTT");
        assert_eq!(input.qual.get(&data), b"IIIIIIIIIIII222");
    }
    #[test]
    fn test_reverse_owned() {
        let mut input = get_owned();
        input.seq.prefix(b"T", &[]);
        input.qual.prefix(b"2", &[]);
        input.seq.reverse(&mut []);
        input.qual.reverse(&mut []);
        assert_eq!(input.qual.get(&[]), b"IIIIIIIIIIII2");
        assert_eq!(input.seq.get(&[]), b"TGCATGCATGCAT");
    }
    #[test]
    fn test_reverse_local() {
        let (mut input, mut data) = get_local();
        input.seq.prefix(b"T", &data);
        input.qual.prefix(b"2", &data);
        input.seq.reverse(&mut data);
        input.qual.reverse(&mut data);
        assert_eq!(input.seq.get(&data), b"TGCATGCATGCAT");
        assert_eq!(input.qual.get(&data), b"IIIIIIIIIIII2");
    }

    fn get_owned2(seq: &[u8]) -> FastQRead {
        FastQRead {
            name: FastQElement::Owned(b"Name".to_vec()),
            seq: FastQElement::Owned(seq.to_vec()),
            qual: FastQElement::Owned(vec![b'I'; seq.len()]),
        }
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
            data.to_vec(),
        );
        assert_eq!(res.0.seq.get(&res.1), seq);
        assert_eq!(res.0.qual.get(&res.1), vec![b'I'; seq.len()]);
        assert_eq!(res.0.name.get(&res.1), b"Name");
        res
    }

    #[test]
    fn test_trimm_poly_n_local() {
        fn trim(seq: &str, min_length: usize, max_mismatch_fraction: f32, base: u8) -> String {
            let (mut read, mut data) = get_local2(seq.as_bytes());
            let mut read2 = WrappedFastQReadMut(&mut read, &mut data);
            read2.trim_poly_base(min_length, max_mismatch_fraction, 5, base);
            std::str::from_utf8(read2.seq()).unwrap().to_string()
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
    fn test_trimm_poly_n() {
        fn trim(seq: &str, min_length: usize, max_mismatch_fraction: f32, base: u8) -> String {
            let mut read = get_owned2(seq.as_bytes());
            let mut data = Vec::new();
            let mut read2 = WrappedFastQReadMut(&mut read, &mut data);
            read2.trim_poly_base(min_length, max_mismatch_fraction, 5, base);
            std::str::from_utf8(read2.seq()).unwrap().to_string()
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
}
