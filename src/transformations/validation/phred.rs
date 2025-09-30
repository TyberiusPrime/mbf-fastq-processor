#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::{apply_in_place_wrapped_plus_all, Step};
use crate::{
    config::{SegmentIndexOrAll, SegmentOrAll},
    demultiplex::Demultiplexed,
};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
enum PhredEncoding {
    #[serde(alias = "sanger")]
    Sanger, //33..=126, offset 33
    #[serde(alias = "solexa")]
    Solexa, //59..=126, offset 64
    #[serde(alias = "illumina")]
    Illumina, //64..=126, offset 64
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidatePhred {
    encoding: PhredEncoding,
    #[serde(default)]
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
}

impl Step for ValidatePhred {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let mut res = Ok(());
        let (lower, upper) = match self.encoding {
            // https://pmc.ncbi.nlm.nih.gov/articles/PMC2847217/
            PhredEncoding::Sanger => (33, 126),
            PhredEncoding::Solexa => (59, 126),
            PhredEncoding::Illumina => (64, 126),
        };
        apply_in_place_wrapped_plus_all(
            self.segment_index.unwrap(),
            |read| {
                if res.is_ok() && read.qual().iter().any(|x| *x < lower || *x > upper) {
                    res = Err(anyhow::anyhow!(
                        "Invalid phred quality found. Expected {lower}..={upper} ({}..={}) : {:?} Bytes: {:?}",
                        lower as char,
                        upper as char,
                        std::str::from_utf8(read.name()),
                        read.qual()
                    ));
                }
            },
            &mut block,
        );
        match res {
            Ok(()) => Ok((block, true)),
            Err(e) => Err(e),
        }
    }
}
