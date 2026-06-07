use bstr::{BStr, BString};
use fastqrab_dna::dna::Hit;
use smallvec::{SmallVec, smallvec};
use std::{borrow::Borrow, num::NonZero, ops::Range};
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
    pub fn new_empty() -> Self {
        FastQChunk {
            names: StringPod::new_all_empty(0),
            seq_quals: DualStringPodBuilder::with_capacity(0, 0).finish(),
            pluses: StringPod::new_all_empty(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
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

    pub fn split_interleaved(self, n: NonZero<usize>) -> Vec<FastQChunk> {
        //TODO: what happens if count % n isn't 0?
        todo!();
        // let split_names = self.names.split_interleaved(n);
        // let split_seq_quals = self.seq_quals.split_interleaved(n);
        // let split_pluses = self.pluses.split_interleaved(n);
        // let mut result = Vec::with_capacity(split_names.len());
        // for (n, sq, p) in split_names
        //     .into_iter()
        //     .zip(split_seq_quals)
        //     .zip(split_pluses)
        // {
        //     result.push(FastQChunk {
        //         names: n,
        //         seq_quals: sq,
        //         pluses: p,
        //         first_read_sequential_number: self.first_read_sequential_number,
        //     })
        // }
        //result
    }

    // pub fn push(&mut self, read: &OwnedFastQRead) {
    //     self.names.push(read.name.as_ref());
    //     self.seq_quals.seq_mut().push(read.seq.as_ref());
    //     self.seq_quals.qual_mut.push(read.qual.as_ref());
    //     self.pluses.push(read.plus.as_ref());
    // }
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

impl<'a> From<FastQRead<'a>> for OwnedFastQRead {
    fn from(r: FastQRead<'a>) -> Self {
        OwnedFastQRead {
            name: r.name.to_owned(),
            seq: r.seq.to_owned(),
            qual: r.qual.to_owned(),
            plus: r.plus.to_owned(),
        }
    }
}

impl<'a> From<&FastQRead<'a>> for OwnedFastQRead {
    fn from(r: &FastQRead<'a>) -> Self {
        r.to_owned()
    }
}

impl FastQRead<'_> {
    pub fn to_owned(&self) -> OwnedFastQRead {
        OwnedFastQRead {
            name: self.name.to_owned(),
            seq: self.seq.to_owned(),
            qual: self.qual.to_owned(),
            plus: self.plus.to_owned(),
        }
    }
}

impl CrossPods for FastQChunk {
    type Companion<'a> = FastQRead<'a>;
    type CompanionMut<'a> = FastQReadMut<'a>;

    // Fixed order: name (col 0), seq_qual (cols 1 & 2: seq then qual), plus (col 3).
    fn pods(&self) -> SmallVec<[PodRef<'_>; 4]> {
        smallvec![
            PodRef::Single(&self.names),
            PodRef::Dual(&self.seq_quals),
            PodRef::Single(&self.pluses),
        ]
    }

    fn pods_mut(&mut self) -> SmallVec<[PodMut<'_>; 4]> {
        smallvec![
            PodMut::Single(&mut self.names),
            PodMut::Dual(&mut self.seq_quals),
            PodMut::Single(&mut self.pluses),
        ]
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

    /// Ensure this segment owns its byte buffers outright (copy-on-write only
    /// where shared).
    pub fn make_exclusive(&mut self) {
        CrossPods::make_exclusive(self);
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

    /// Number of segments contributing a read to this molecule.
    #[must_use]
    pub fn len(&self) -> usize {
        self.reads.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reads.is_empty()
    }

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

impl From<FastQRead<'_>> for OwnedMolecule {
    fn from(read: FastQRead<'_>) -> Self {
        OwnedMolecule {
            reads: smallvec![read.to_owned()],
        }
    }
}

impl From<MoleculeMut<'_>> for OwnedMolecule {
    fn from(molecule: MoleculeMut<'_>) -> Self {
        OwnedMolecule {
            reads: molecule
                .into_iter()
                .map(|read| OwnedFastQRead {
                    name: read.name.to_owned(),
                    seq: read.seq.to_owned(),
                    qual: read.qual.to_owned(),
                    plus: read.plus.to_owned(),
                })
                .collect(),
        }
    }
}

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
            
        

pub fn hit_to_qualities(molecule: Molecule, hits: &SmallVec<[Hit; 1]>) -> BString {
    let mut qual = BString::new(Vec::new());
    for hit in hits.iter() {
        qual.extend_from_slice(
            &molecule[hit.segment_index.as_index()].seq[(hit.loc_start as usize)..(hit.loc_start + 
        hit.loc_len as u32) as usize]);
    }
    qual
}

impl<'a> Iterator for Molecules<'a> {
    type Item = Molecule<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.members.is_empty() {
            return None;
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.members
            .first()
            .map_or((0, Some(0)), RowCompanions::size_hint)
    }
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
            return None;
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.members
            .first()
            .map_or((0, Some(0)), CrossPodsRecordsMut::size_hint)
    }
}

impl ExactSizeIterator for MoleculesMut<'_> {}
