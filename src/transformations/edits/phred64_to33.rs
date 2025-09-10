#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::Step;
use crate::demultiplex::Demultiplexed;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Phred64To33 {}

impl Step for Phred64To33 {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        block.apply_mut(|segments| {
            for read in segments {
                let qual = read.qual();
                let new_qual: Vec<_> = qual.iter().map(|x| x.saturating_sub(31)).collect();
                assert!(
                    !new_qual.iter().any(|x| *x < 33),
                    "Phred 64-33 conversion yielded values below 33 -> wasn't Phred 64 to begin with"
                );
                read.replace_qual(new_qual);
            }
        });
        //no tag change.
        (block, true)
    }
}
