#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

use super::extract_numeric_tags_plus_all;

/// Convert read length into a tag

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Length {
    pub out_label: String,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialLength {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl Step for Length {
    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Numeric,
        ))
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        extract_numeric_tags_plus_all(
            self.segment,
            &self.out_label,
            #[allow(clippy::cast_precision_loss)]
            |read| read.seq().len() as f64,
            #[allow(clippy::cast_precision_loss)]
            |reads| {
                let total_length: usize = reads.iter().map(|read| read.seq().len()).sum();
                total_length as f64
            },
            &mut block,
        );

        Ok((block, true))
    }
}
