use anyhow::{Result, bail};
use bstr::{BStr, BString, ByteSlice};
use smallvec::SmallVec;
use std::{num::NonZero, ops::Range};
use stringpod::{
    CrossPods, CrossPodsRecordsMut, DualStringPod, DualStringPodBuilder, PodMut, PodRef,
    RowCompanions, StringPod, StringPodBuilder,
};

//What our parsers output (eventually)
#[derive(Debug, Clone)]
pub struct FastQChunk {
    /// Read names
    pub names: StringPod,
    //read sequences and qualties
    //always the same length
    pub seq_quals: DualStringPod,
    //the data from the '+' lines.
    pub pluses: StringPod,
}

impl FastQChunk {
    #[must_use]
    pub fn new_empty() -> Self {
        FastQChunk {
            names: StringPod::new_all_empty(0),
            seq_quals: DualStringPodBuilder::with_capacity(0, 0).finish(),
            pluses: StringPod::new_all_empty(0),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Build a segment from a sequence of owned reads — e.g. reconstructing a
    /// block after out-of-band buffering (reservoir sampling). Each read
    /// contributes one row across the name / seq+qual / plus columns.
    pub fn from_owned_reads<'a, I>(reads: I) -> Self
    where
        I: IntoIterator<Item = &'a OwnedFastQRead>,
        I::IntoIter: ExactSizeIterator,
    {
        let reads = reads.into_iter();
        let n = reads.len();
        let mut names = StringPodBuilder::with_capacity(0, n);
        let mut seq_quals = DualStringPodBuilder::with_capacity(0, n);
        let mut pluses = StringPodBuilder::with_capacity(0, n);
        for read in reads {
            names.push(read.name.as_ref());
            seq_quals.push(read.seq.as_ref(), read.qual.as_ref());
            pluses.push(read.plus.as_ref());
        }
        FastQChunk {
            names: names.finish(),
            seq_quals: seq_quals.finish(),
            pluses: pluses.finish(),
        }
    }

    /// Deinterleave this segment into `n` segments. Read `i` is routed to output
    /// segment `i % n`, so segment `j` collects reads `j, j+n, j+2n, …`. This is
    /// the inverse of interleaving `n` lockstep streams round-robin (e.g.
    /// splitting an interleaved R1/R2 file back into separate mates with `n == 2`).
    ///
    /// Each output is freshly built, copying the relevant entries out of `self`'s
    /// columns; the originals are consumed.
    ///
    /// # Panics
    /// If `self.len()` is not a multiple of `n` — every output segment must end
    /// up with the same number of reads.
    pub fn split_interleaved(self, n: NonZero<usize>) -> Result<Vec<FastQChunk>> {
        let n = n.get();
        let total = self.len();
        if !total.is_multiple_of(n) {
            bail!(
                "Can't split {total} reads into {n} interleaved segments.\n\
            Since the config verifies that your input.config.block_size satisfies this,\n\
            this means that your total read count is not compatible with this split"
            );
        }
        let per_segment = total / n;

        let mut names: Vec<StringPodBuilder> = (0..n)
            .map(|_| StringPodBuilder::with_capacity(0, per_segment))
            .collect();
        let mut seq_quals: Vec<DualStringPodBuilder> = (0..n)
            .map(|_| DualStringPodBuilder::with_capacity(0, per_segment))
            .collect();
        let mut pluses: Vec<StringPodBuilder> = (0..n)
            .map(|_| StringPodBuilder::with_capacity(0, per_segment))
            .collect();

        for i in 0..total {
            let seg = i % n;
            names[seg].push(self.names.get(i).as_bytes());
            let (seq, qual) = self.seq_quals.pair(i);
            seq_quals[seg].push(seq.as_bytes(), qual.as_bytes());
            pluses[seg].push(self.pluses.get(i).as_bytes());
        }

        Ok(names
            .into_iter()
            .zip(seq_quals)
            .zip(pluses)
            .map(|((names, seq_quals), pluses)| FastQChunk {
                names: names.finish(),
                seq_quals: seq_quals.finish(),
                pluses: pluses.finish(),
            })
            .collect())
    }

    // pub fn push(&mut self, read: &OwnedFastQRead) {
    //     self.names.push(read.name.as_ref());
    //     self.seq_quals.seq_mut().push(read.seq.as_ref());
    //     self.seq_quals.qual_mut.push(read.qual.as_ref());
    //     self.pluses.push(read.plus.as_ref());
    // }
    //
    //

    /// Length of all sequences, added up
    #[must_use]
    pub fn total_seq_len(&self) -> usize {
        self.seq_quals.iter_seq_lens().sum()
    }
}

#[derive(Debug)]
pub struct FastQRead<'a> {
    pub name: &'a BStr,
    pub seq: &'a BStr,
    pub qual: &'a BStr,
    pub plus: &'a BStr,
}

#[derive(Debug)]
pub struct FastQReadMut<'a> {
    pub name: &'a mut BStr,
    pub seq: &'a mut BStr,
    pub qual: &'a mut BStr,
    pub plus: &'a mut BStr,
}

#[derive(Debug, Clone)]
pub struct OwnedFastQRead {
    pub name: BString,
    pub seq: BString,
    pub qual: BString,
    pub plus: BString,
}

// impl<'a> From<FastQRead<'a>> for OwnedFastQRead {
//     fn from(r: FastQRead<'a>) -> Self {
//         OwnedFastQRead {
//             name: r.name.to_owned(),
//             seq: r.seq.to_owned(),
//             qual: r.qual.to_owned(),
//             plus: r.plus.to_owned(),
//         }
//     }
// }
//
// impl<'a> From<&FastQRead<'a>> for OwnedFastQRead {
//     fn from(r: &FastQRead<'a>) -> Self {
//         r.to_owned()
//     }
// }

impl FastQRead<'_> {
    #[must_use]
    pub fn to_owned(&self) -> OwnedFastQRead {
        OwnedFastQRead {
            name: self.name.to_owned(),
            seq: self.seq.to_owned(),
            qual: self.qual.to_owned(),
            plus: self.plus.to_owned(),
        }
    }

    pub fn append_as_fastq(&self, out: &mut Vec<u8>) {
        let name = self.name;
        let seq = self.seq;
        let qual = self.qual;
        out.push(b'@');

        out.extend(name.as_bytes());

        out.push(b'\n');
        out.extend(seq.as_bytes());

        out.extend(b"\n+\n");
        out.extend(qual.as_bytes());
        out.push(b'\n');
    }

    pub fn append_as_fasta(&self, out: &mut Vec<u8>) {
        let name = self.name;
        let seq = self.seq;
        out.push(b'>');
        out.extend(name.as_bytes());
        out.push(b'\n');
        out.extend(seq.as_bytes());
        out.push(b'\n');
    }
}

impl FastQReadMut<'_> {
    pub fn reverse_complement_iupac(&mut self) {
        self.qual.reverse();
        let new_seq = fastqrab_dna::dna::reverse_complement_iupac(self.seq.as_ref());
        for (a, b) in self.seq.iter_mut().zip(new_seq.iter()) {
            *a = *b;
        }
    }
}

impl CrossPods for FastQChunk {
    type Companion<'a> = FastQRead<'a>;
    type CompanionMut<'a> = FastQReadMut<'a>;

    // Fixed order: name (col 0), seq_qual (cols 1 & 2: seq then qual), plus (col 3).
    fn pods(&self) -> SmallVec<[PodRef<'_>; 4]> {
        // Built with explicit push (not smallvec![]) so llvm-cov doesn't attribute
        // the macro's dead heap-spill branch to these lines. push auto-spills too.
        let mut pods = SmallVec::new();
        pods.push(PodRef::Single(&self.names));
        pods.push(PodRef::Dual(&self.seq_quals));
        pods.push(PodRef::Single(&self.pluses));
        pods
    }

    fn pods_mut(&mut self) -> SmallVec<[PodMut<'_>; 4]> {
        let mut pods = SmallVec::new();
        pods.push(PodMut::Single(&mut self.names));
        pods.push(PodMut::Dual(&mut self.seq_quals));
        pods.push(PodMut::Single(&mut self.pluses));
        pods
    }

    fn to_companion<'a>(parts: &[&'a BStr]) -> FastQRead<'a> {
        FastQRead {
            name: parts[0],
            seq: parts[1],
            qual: parts[2],
            plus: parts[3],
        }
    }

    fn to_companion_mut(parts: SmallVec<[&mut BStr; 4]>) -> FastQReadMut<'_> {
        let mut it = parts.into_iter();
        FastQReadMut {
            name: it.next().expect("name part"),
            seq: it.next().expect("seq part"),
            qual: it.next().expect("qual part"),
            plus: it.next().expect("plus part"),
        }
    }
}

/// Count-changing operations on a single segment. [`FastQBlocksCombined`] fans
/// these out across every segment at once so the segments stay in lockstep (same
/// number of reads).
///
/// [`FastQBlocksCombined`]: crate::io::FastQBlocksCombined
impl FastQChunk {
    /// Number of reads in this segment.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.len()
    }

    /// A segment over reads `range`, sharing this segment's byte buffers — no
    /// bytes are copied (O(1) for fixed-length columns, O(range) metadata for
    /// variable). The combiner uses this to align segment block sizes by
    /// slicing each segment's current block down to the common read count
    /// instead of re-copying reads into fixed-size blocks.
    ///
    /// # Panics
    /// If `range.start > range.end` or `range.end > self.len()`.
    #[must_use]
    pub fn slice(&self, range: Range<usize>) -> FastQChunk {
        let n = u32::try_from(range.end - range.start).expect("slice length exceeds u32::MAX");
        FastQChunk {
            names: self.names.slice(range.clone()),
            seq_quals: self.seq_quals.slice(range),
            pluses: StringPod::new_all_empty(n),
        }
    }

    /// Keep only the first `n` reads.
    pub fn truncate(&mut self, n: usize) {
        self.names.truncate(n);
        self.seq_quals.truncate(n);
        self.pluses.truncate(n);
    }

    /// Remove the reads in `range`.
    pub fn drain(&mut self, range: Range<usize>) {
        self.names.drain(range.clone());
        self.seq_quals.drain(range.clone());
        self.pluses.drain(range);
    }

    /// Keep read `i` iff `keep[i]`.
    pub fn retain_by_bools(&mut self, keep: &[bool]) {
        self.names.retain_by_bools(keep);
        self.seq_quals.retain_by_bools(keep);
        self.pluses.retain_by_bools(keep);
    }

    /// Iterate `(read_index_within_block, read)` pairs, optionally restricted to the reads
    /// carrying a given demultiplex tag.
    ///
    /// - `target_tag == None` and `output_tags == None`: yield every read.
    /// - `target_tag == Some(t)` and `output_tags == Some(tags)`: yield only the
    ///   reads whose entry in `tags` equals `t`. The yielded index is the read's
    ///   position within the full block, not within the filtered subset.
    ///
    /// # Panics
    /// If exactly one of `target_tag` / `output_tags` is set — supplying a tag to
    /// filter to without the per-read tags (or vice versa) is a contract
    /// violation.
    #[must_use]
    pub fn iter_filtered_to_tag<'a>(
        &'a self,
        target_tag: Option<crate::io::reads::DemultiplexTag>,
        output_tags: Option<&'a Vec<crate::io::reads::DemultiplexTag>>,
    ) -> Box<dyn Iterator<Item = (usize, FastQRead<'a>)> + 'a> {
        match (target_tag, output_tags) {
            (Some(filter_to), Some(per_read_tags)) => Box::new(
                self.iter()
                    .zip(per_read_tags)
                    .enumerate()
                    .filter(move |(_read_idx, (_read, dt))| **dt == filter_to)
                    .map(|(read_idx, (read, _dt))| (read_idx, read)),
            ),
            (None, None) => Box::new(self.iter().enumerate()),
            _ => {
                //cov:excl-start
                panic!(
                    "iter_filtered_to_tag: target_tag and output_tags must both be set or both be None; got {target_tag:?}, {output_tags:?}"
                )
                //cov:excl-stop
            }
        }
    }
}

/// A molecule: read `i` drawn from every segment of a block, in segment order.
/// The read-only form, yielded by [`molecules`].
pub type Molecule<'a> = SmallVec<[FastQRead<'a>; 4]>;

/// A mutable molecule: read `i`'s mutable view from every segment, in segment
/// order. Yielded by [`molecules_mut`].
pub type MoleculeMut<'a> = SmallVec<[FastQReadMut<'a>; 4]>;

/// An owned molecule: one [`OwnedFastQRead`] per segment, in segment order.
/// Suitable for stashing in a `Vec` outside the block's lifetime (e.g.
/// reservoir sampling).
#[derive(Debug, Clone)]
pub struct OwnedMolecule {
    /// One owned read per segment, in segment order.
    pub reads: SmallVec<[OwnedFastQRead; 4]>,
}

impl OwnedMolecule {
    //cov:excl-start - these are just for completneses
    /// Number of segments contributing a read to this molecule.
    #[mutants::skip]
    #[must_use]
    pub fn len(&self) -> usize {
        self.reads.len()
    }

    #[mutants::skip]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reads.is_empty()
    }
    //cov:excl-stop

    /// The read from segment `index`.
    ///
    /// # Panics
    /// If `index >= self.len()`.
    #[must_use]
    pub fn get(&self, index: usize) -> &OwnedFastQRead {
        &self.reads[index]
    }
}

impl From<Molecule<'_>> for OwnedMolecule {
    fn from(molecule: Molecule<'_>) -> Self {
        OwnedMolecule {
            reads: molecule.iter().map(FastQRead::to_owned).collect(),
        }
    }
}

impl From<&FastQRead<'_>> for OwnedMolecule {
    fn from(read: &FastQRead<'_>) -> Self {
        let mut reads = SmallVec::new();
        reads.push(read.to_owned());
        OwnedMolecule { reads }
    }
}

// impl From<MoleculeMut<'_>> for OwnedMolecule {
//     fn from(molecule: MoleculeMut<'_>) -> Self {
//         OwnedMolecule {
//             reads: molecule
//                 .into_iter()
//                 .map(|read| OwnedFastQRead {
//                     name: read.name.to_owned(),
//                     seq: read.seq.to_owned(),
//                     qual: read.qual.to_owned(),
//                     plus: read.plus.to_owned(),
//                 })
//                 .collect(),
//         }
//     }
// }

/// Iterate molecules over a slice of lockstep segments: each item is read `i`
/// drawn from every segment, in segment order. See [`Molecules`].
///
/// # Panics
/// If a segment's columns disagree on read count (same contract as
/// [`CrossPods::iter`]).
#[must_use]
pub fn molecules(segments: &[FastQChunk]) -> Molecules<'_> {
    Molecules {
        members: segments.iter().map(CrossPods::iter).collect(),
    }
}

/// Iterate molecules mutably over a slice of lockstep segments. Shared buffers
/// are made exclusive first (silent copy-on-write). See [`MoleculesMut`].
///
/// # Panics
/// If a segment's columns disagree on read count.
#[must_use]
pub fn molecules_mut(segments: &mut [FastQChunk]) -> MoleculesMut<'_> {
    MoleculesMut {
        members: segments.iter_mut().map(CrossPods::iter_mut).collect(),
    }
}

/// Iterator over a block's molecules. Created by [`molecules`].
pub struct Molecules<'a> {
    members: SmallVec<[RowCompanions<'a, FastQChunk>; 4]>,
}

impl<'a> Iterator for Molecules<'a> {
    type Item = Molecule<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.members.is_empty() {
            return None; //cov:excl-line
        }
        let mut molecule = SmallVec::with_capacity(self.members.len());
        for (index, member) in self.members.iter_mut().enumerate() {
            match member.next() {
                Some(read) => molecule.push(read),
                None if index == 0 => return None,
                None => unreachable!("FastQ segments fell out of lockstep during iteration"), //cov:excl-line
            }
        }
        Some(molecule)
    }

    //cov:excl-start
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.members
            .first()
            .map_or((0, Some(0)), RowCompanions::size_hint)
    }
    //cov:excl-stop
}

impl ExactSizeIterator for Molecules<'_> {}

/// Iterator over a block's molecules, mutably. Created by [`molecules_mut`].
pub struct MoleculesMut<'a> {
    members: SmallVec<[CrossPodsRecordsMut<'a, FastQChunk>; 4]>,
}

impl<'a> Iterator for MoleculesMut<'a> {
    type Item = MoleculeMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.members.is_empty() {
            unreachable!("A 0 reads molecule is unexpected");
        }
        let mut molecule = SmallVec::with_capacity(self.members.len());
        for (index, member) in self.members.iter_mut().enumerate() {
            match member.next() {
                Some(read) => molecule.push(read),
                None if index == 0 => return None,
                None => panic!("FastQ segments fell out of lockstep during iteration"),
            }
        }
        Some(molecule)
    }

    //cov:excl-start
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.members
            .first()
            .map_or((0, Some(0)), CrossPodsRecordsMut::size_hint)
    }
    //cov:excl-stop
}

impl ExactSizeIterator for MoleculesMut<'_> {}

/// Splits a read 'name' into the actual name/id and the comment
#[must_use]
pub fn split_name_and_comment(name: &BStr, read_comment_insert_char: u8) -> (&BStr, &BStr) {
    use bstr::ByteSlice;
    //let pos_of_first_space = name.iter().position(|&x| x == read_comment_insert_char);
    match name.find_byte(read_comment_insert_char) {
        Some(pos) => (name[..pos].as_ref(), name[pos + 1..].as_ref()),
        None => (name, BStr::new("")),
    }
}

/// Splits a read 'name' into the actual name/id and the comment, mutably.
pub fn split_name_and_comment_mut(
    name: &mut BStr,
    read_comment_insert_char: u8,
) -> (&mut BStr, &mut BStr) {
    use bstr::ByteSlice;
    match name.find_byte(read_comment_insert_char) {
        Some(pos) => {
            let (left, right) = name.split_at_mut(pos);
            (left.as_bstr_mut(), right[1..].as_bstr_mut())
        }
        None => (name, <&mut BStr>::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(i: usize) -> OwnedFastQRead {
        // seq and qual must share a per-entry length (DualStringPod invariant).
        OwnedFastQRead {
            name: BString::from(format!("r{i}")),
            seq: BString::from(format!("SEQ{i}")),
            qual: BString::from(format!("QAL{i}")),
            plus: BString::from(format!("plus{i}")),
        }
    }

    #[test]
    fn split_interleaved_routes_round_robin() {
        // 6 reads, deinterleave into 2 streams: segment 0 -> r0,r2,r4; seg 1 -> r1,r3,r5.
        let reads: Vec<_> = (0..6).map(read).collect();
        let chunk = FastQChunk::from_owned_reads(&reads);

        let segments = chunk.split_interleaved(NonZero::new(2).unwrap()).unwrap();
        assert_eq!(segments.len(), 2);

        let names =
            |seg: &FastQChunk| -> Vec<String> { seg.iter().map(|r| r.name.to_string()).collect() };
        assert_eq!(names(&segments[0]), vec!["r0", "r2", "r4"]);
        assert_eq!(names(&segments[1]), vec!["r1", "r3", "r5"]);

        // Seq/qual/plus columns travel with their read.
        let r2 = segments[0].iter().nth(1).unwrap();
        assert_eq!(r2.seq.to_string(), "SEQ2");
        assert_eq!(r2.qual.to_string(), "QAL2");
        assert_eq!(r2.plus.to_string(), "plus2");
    }

    #[test]
    fn split_interleaved_n1_is_identity() {
        let reads: Vec<_> = (0..3).map(read).collect();
        let chunk = FastQChunk::from_owned_reads(&reads);
        let segments = chunk.split_interleaved(NonZero::new(1).unwrap()).unwrap();
        assert_eq!(segments.len(), 1);
        let got: Vec<String> = segments[0].iter().map(|r| r.name.to_string()).collect();
        assert_eq!(got, vec!["r0", "r1", "r2"]);
    }

    #[test]
    #[should_panic(expected = "Can't split 5 reads into 2 interleaved segments")]
    fn split_interleaved_uneven_panics() {
        let reads: Vec<_> = (0..5).map(read).collect();
        let chunk = FastQChunk::from_owned_reads(&reads);
        let _ = chunk.split_interleaved(NonZero::new(2).unwrap()).unwrap();
    }
}
