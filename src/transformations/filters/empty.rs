#![allow(clippy::unnecessary_wraps)] //eserde false positives

use super::super::{Step, SegmentOrAll};
use crate::demultiplex::Demultiplexed;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Empty {
    #[eserde(compat)]
    pub segment: SegmentOrAll,
}

impl Step for Empty {
    fn apply(
        &mut self,
        mut _block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        unreachable!("Should have been replaced before validation");
    }
}
