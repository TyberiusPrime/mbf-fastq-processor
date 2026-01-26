#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use crate::config::deser::{FromTomlTable, bstring_from_string};

/// Validate that the sequence is only consisting of the specified bases
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateSeq {
    #[serde(deserialize_with = "bstring_from_string")]
    #[schemars(with = "String")]
    pub allowed: BString,

    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>, //todo: remove option
}
impl FromTomlTableNested for ValidateSeq {
    fn from_toml_table(table: &toml_edit::Table, mut helper: TableErrorHelper) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let allowed = helper.get("allowed");
        let segment_index: TomlResult<SegmentIndexOrAll> = helper.get_segment_all(true);
        helper.deny_unknown()?;

        Ok(ValidateSeq {
            allowed: allowed?,
            segment_index: Some(segment_index?),
        })
    }
}

impl Step for ValidateSeq {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        Ok(())
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut res = Ok(());
        block.apply_in_place_wrapped_plus_all(
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |read| {
                if res.is_ok() && read.seq().iter().any(|x| !self.allowed.contains(x)) {
                    res = Err(anyhow::anyhow!(
                        "Invalid base found in read named '{}', sequence: '{}' Bytes: {:?}",
                        BString::from(read.name()),
                        BString::from(read.seq()),
                        read.seq()
                    ));
                }
            },
            None,
        );
        match res {
            Ok(()) => Ok((block, true)),
            Err(e) => Err(e),
        }
    }
}
