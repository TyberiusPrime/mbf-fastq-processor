#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::dna::HitRegion;

/// Cut a fixed number of bases from the start of reads
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct CutStart {
    n: usize,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,
    #[tpd(default)]
    if_tag: Option<String>,
}

impl VerifyIn<PartialConfig> for PartialCutStart {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
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

impl Step for CutStart {
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
    //to modify location tags
    fn must_see_all_tags(&self) -> bool {
        true
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
            |read| read.cut_start(self.n),
            condition.as_deref(),
        );

        block.filter_tag_locations(
            self.segment,
            |location: &HitRegion, _pos, _seq, _read_len: usize| -> NewLocation {
                if location.start < self.n {
                    NewLocation::Remove
                } else {
                    NewLocation::New(HitRegion {
                        start: location.start - self.n,
                        len: location.len,
                        segment_index: location.segment_index,
                    })
                }
            },
            condition.as_deref(),
        );

        Ok((block, true))
    }
}
