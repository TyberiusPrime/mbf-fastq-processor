#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

/// Cut a fixed number of bases from the end of reads
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct CutEnd {
    n: usize,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,
    if_tag: Option<String>,
}

impl VerifyIn<PartialConfig> for PartialCutEnd {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        self.n.verify(|v| {
            if *v == 0 {
                Err(ValidationFailure::new(
                    "n must be > 0",
                    Some("Set to a positive integer."),
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl Step for CutEnd {
    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        self.if_tag.as_ref().map(|tag_str| {
            let cond_tag = ConditionalTag::from_string(tag_str.clone());
            vec![(
                cond_tag.tag.clone(),
                &[
                    TagValueType::Bool,
                    TagValueType::String,
                    TagValueType::Location,
                ][..],
            )]
        })
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let condition = self.if_tag.as_ref().map(|tag_str| {
            let cond_tag = ConditionalTag::from_string(tag_str.clone());
            get_bool_vec_from_tag(&block, &cond_tag)
        });

        block.apply_in_place(
            self.segment,
            |read| read.cut_end(self.n),
            condition.as_deref(),
        );
        block.filter_tag_locations_beyond_read_length(self.segment);

        Ok((block, true))
    }

    //to cut location tags
    fn must_see_all_tags(&self) -> bool {
        true
    }
}
