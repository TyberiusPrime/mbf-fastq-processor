use bstr::BStr;
use std::{num::NonZero, ops::Range};
use smallvec::{SmallVec, smallvec};
use stringpod::{CrossPods, CrossPodRecords, CrossPodsRecordsMut, CrossPodLocations, DualStringPod, DualStringPodBuilder, PodMut, PodRef, StringPod};

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
    pub first_read_sequential_number: usize,
}

impl FastQChunk {

    pub fn new_empty()-> Self  {
        FastQChunk {
            names: StringPod::new_all_empty(0),
            seq_quals: DualStringPodBuilder::with_capacity(0,0).finish(),
            pluses: StringPod::new_all_empty(0),
            first_read_sequential_number: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
    pub fn len(&self) -> usize {
        self.names.len()
    }


    pub fn truncate(&mut self, n: usize)  {
        self.names.truncate(n);
        self.seq_quals.truncate(n);
        self.pluses.truncate(n);
    }

    pub fn drain(&mut self, range: Range<usize>) {
        self.names.drain(range.clone());
        self.seq_quals.drain(range.clone());
        self.pluses.drain(range);
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

    pub fn retain_by_bools(&mut self, keep: &[bool]) {
        self.names.retain_by_bools(keep);
        self.seq_quals.retain_by_bools(keep);
        self.pluses.retain_by_bools(keep);
    }

    pub fn iter<'a>(&'a self) -> CrossPodRecords<'a, Self> {
        //let locs = CrossPodLocations::per_row(self);
        //locs.into_iter(self)
        todo!();
    }


    pub fn to_exclusive(self) -> Self {
        Self {
            names: self.names.to_exclusive(),
            seq_quals: self.seq_quals.to_exclusive(),
            pluses: self.pluses.to_exclusive(),
            first_read_sequential_number: self.first_read_sequential_number,

        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> CrossPodsRecordsMut<'a, Self> {
        //let locs = CrossPodLocations::per_row(self);
        //locs.try_iter_mut(self).expect("Make sure you have exclusive use by using .to_exclusive() before")
        todo!();
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
